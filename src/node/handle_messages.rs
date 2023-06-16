
use crate::node::*;

use super::peer_comunication::block_downloader::send_get_data_message_for_blocks;

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
pub fn handle_inv_message(stream: &mut TcpStream, msg_bytes: Vec<u8>, safe_headers: &SafeVecHeader, safe_blockchain: &SafeBlockChain, safe_pending_tx: &SafePendingTx, logger: &Logger) -> Result<(), NodeError> {
    let inv_msg = match InvMessage::from_bytes(&msg_bytes) {
        Ok(msg) => msg,
        Err(_) => return Err(NodeError::ErrorRecevingBroadcastedInventory),
    };

    let block_hashes = inv_msg.get_block_hashes();
    let transaction_hashes = inv_msg.get_transaction_hashes();

    let mut request_block_hashes = Vec::new();
    let mut request_transaction_hashes = Vec::new();

    match safe_blockchain.lock(){
        Ok(blockchain) => {
            for hash in block_hashes{
                if !blockchain.contains_key(&hash){
                    request_block_hashes.push(hash);
                }
            }
        },
        Err(_) => return Err(NodeError::ErrorSharingReference),
    };

    match safe_pending_tx.lock(){
        Ok(pending_tx) => {
            for hash in transaction_hashes{
                if !pending_tx.contains_key(&hash){
                    request_transaction_hashes.push(hash);
                }
            }
        },
        Err(_) => return Err(NodeError::ErrorSharingReference),
    };
    
    send_get_data_message_for_blocks(request_block_hashes, stream).map_err(|_| NodeError::ErrorDownloadingBlockBundle)?;
    send_get_data_message_for_transactions(request_transaction_hashes, stream)
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

/// Sends a getdata message to the stream, requesting the blocks with the specified hashes.
/// Returns an error if it was not possible to send the message.
pub fn send_get_data_message_for_transactions(
    hashes: Vec<[u8; 32]>,
    stream: &mut TcpStream,
) -> Result<(), NodeError> {
    let get_data_message = GetDataMessage::create_message_inventory_transaction_type(hashes);

    match get_data_message.send_to(stream) {
        Ok(_) => Ok(()),
        Err(_) => Err(NodeError::ErrorGettingTx),
    }
}
