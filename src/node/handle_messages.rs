use crate::node::*;

///Handles the headers message by hashing the last received header and asking for more headers.
pub fn handle_block_headers_message(msg_bytes: Vec<u8>, block_headers: &mut Vec<BlockHeader>) -> Result<(), NodeError> {
    let block_headers_msg = match BlockHeadersMessage::from_bytes(&msg_bytes) {
        Ok(block_headers_message) => block_headers_message,
        Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
    };

    let received_block_headers = block_headers_msg.headers;

    block_headers.extend(received_block_headers);
    Ok(())
}

///Handles the block message by validating the proof of work and the proof of inclusion and the saving it.
/// If the block is already in the blockchain, it is not saved.
pub fn handle_block_message(msg_bytes: Vec<u8>,  block_headers: &mut Vec<BlockHeader>, blockchain: &mut Vec<Block>, logger: &Logger, ibd: bool) -> Result<(), NodeError> {
    let block_msg = match BlockMessage::from_bytes(&msg_bytes) {
        Ok(block_msg) => block_msg,
        Err(_) => return Err(NodeError::ErrorReceivingBroadcastedBlock),
    };
    
    if validate_proof_of_work(block_msg.block.get_header()) {
        if validate_proof_of_inclusion(&block_msg.block) {
            blockchain.push(block_msg.block);
            //add_block(block_msg.block, block_headers, blockchain, !ibd)?;
        } else {
            logger.log(String::from("Proof of inclusion failed for a block"));
            return Err(NodeError::ErrorValidatingBlock)
        }
    } else {
        logger.log(String::from("Proof of work failed for a block"));
        return Err(NodeError::ErrorValidatingBlock)
    }

    Ok(())
}

///Handles the inv message by asking for the blocks that are not in the blockchain.
///If the block is already in the blockchain, it is not saved.
pub fn handle_inv_message(stream: &mut TcpStream, msg_bytes: Vec<u8>, block_headers: &mut Vec<BlockHeader>, blockchain: &mut Vec<Block>, logger: &Logger) -> Result<(), NodeError> {
    let inv_msg = match InvMessage::from_bytes(&msg_bytes) {
        Ok(msg) => msg,
        Err(_) => return Err(NodeError::ErrorRecevingBroadcastedInventory),
    };

    match get_blocks_from_bundle(inv_msg.get_block_hashes(), stream, logger) {
        Ok(blocks) => {
            for block in blocks {
                 blockchain.push(block);
            }
            Ok(())
        }
        Err(_) => Err(NodeError::ErrorDownloadingBlockBundle),
    }
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
