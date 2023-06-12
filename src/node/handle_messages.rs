use crate::node::*;

///Handles the headers message by hashing the last received header and asking for more headers.
pub fn handle_block_headers_message(msg_bytes: Vec<u8>, safe_block_headers: &SafeVecHeader) -> Result<(), NodeError> {
    let block_headers_msg = match BlockHeadersMessage::from_bytes(&msg_bytes) {
        Ok(block_headers_message) => block_headers_message,
        Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
    };

    let received_block_headers = block_headers_msg.headers;

    match safe_block_headers.lock(){
        Ok(mut block_headers) => block_headers.extend(received_block_headers),
        Err(_) => return Err(NodeError::ErrorSharingReference),
    }
            
    Ok(())
}

///Handles the block message by validating the proof of work and the proof of inclusion and the saving it.
/// If the block is already in the blockchain, it is not saved.
pub fn handle_block_message(msg_bytes: Vec<u8>, safe_headers: &SafeVecHeader, safe_blockchain: &SafeBlockChain, logger: &Logger, ibd: bool) -> Result<(), NodeError> {
    let block = match BlockMessage::from_bytes(&msg_bytes) {
        Ok(block_msg) => block_msg.block,
        Err(_) => return Err(NodeError::ErrorReceivingBroadcastedBlock),
    };
    let block_header = block.get_header();

    if !validate_proof_of_work(&block_header) {
        logger.log(String::from("Proof of work failed for a block"));
        return Err(NodeError::ErrorValidatingBlock);
    };
    if !validate_proof_of_inclusion(&block) {
        logger.log(String::from("Proof of inclusion failed for a block"));
        return Err(NodeError::ErrorValidatingBlock)
    };

    if !ibd{
        let mut block_headers = safe_headers.lock().map_err(|_| NodeError::ErrorSharingReference)?;
        block_headers.push(block_header);
    }
    let mut blockchain = safe_blockchain.lock().map_err(|_| NodeError::ErrorSharingReference)?;
    blockchain.insert(block.header_hash(),block);
            
    Ok(())
}

///Handles the inv message by asking for the blocks that are not in the blockchain.
///If the block is already in the blockchain, it is not saved.
pub fn handle_inv_message(stream: &mut TcpStream, msg_bytes: Vec<u8>, safe_headers: &SafeVecHeader, safe_blockchain: &SafeBlockChain, logger: &Logger) -> Result<(), NodeError> {
    let inv_msg = match InvMessage::from_bytes(&msg_bytes) {
        Ok(msg) => msg,
        Err(_) => return Err(NodeError::ErrorRecevingBroadcastedInventory),
    };

    let hashes = inv_msg.get_block_hashes();
    let mut request_hashes = Vec::new();
    match safe_blockchain.lock(){
        Ok(blockchain) => {
            for hash in hashes{
                if !blockchain.contains_key(&hash){
                    request_hashes.push(hash);
                }
            }
        },
        Err(_) => return Err(NodeError::ErrorSharingReference),
    };
    
    get_blocks_from_bundle(request_hashes, stream, safe_headers, safe_blockchain, logger).map_err(|_| NodeError::ErrorDownloadingBlockBundle)
}

///Handles the ping message by sending a pong message.
pub fn handle_ping_message(stream: &mut TcpStream, header_message: &HeaderMessage, nonce: Vec<u8>) -> Result<(), NodeError> {
    if nonce.len() != 8 {
        return Err(NodeError::ErrorReceivingPing);
    }

    let mut pong_bytes = header_message.to_bytes();
    pong_bytes.extend(nonce);
    pong_bytes[5] = b'o';

    if stream.write(&pong_bytes).is_err() {
        return Err(NodeError::ErrorSendingPong);
    }
    Ok(())
}
