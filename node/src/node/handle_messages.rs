use crate::node::*;
use crate::node::initial_block_download::HASHEDGENESISBLOCK;
use crate::node::get_block_headers_message::MAX_QUANTITY_FOR_GET_HEADERS;

use super::peer_comunication::block_downloader::send_get_data_message_for_blocks;

///Handles the headers message by hashing the last received header and asking for more headers.
pub fn handle_block_headers_message(
    msg_bytes: Vec<u8>,
    safe_block_headers: &SafeVecHeader,
    safe_headers_index: &SafeHeaderIndex
) -> Result<(), NodeError> {
    let block_headers_msg = match BlockHeadersMessage::from_bytes(&msg_bytes) {
        Ok(block_headers_message) => block_headers_message,
        Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
    };

    let received_block_headers = block_headers_msg.headers;

    insert_new_headers(received_block_headers, &safe_block_headers, &safe_headers_index)?;

    Ok(())
}

/// Handles the block message by validating the proof of work and the proof of inclusion and the saving it.
/// If the block is already in the blockchain, it is not saved.
pub fn handle_block_message(
    msg_bytes: Vec<u8>,
    safe_headers: &SafeVecHeader,
    safe_blockchain: &SafeBlockChain,
    safe_pending_tx: &SafePendingTx,
    safe_headers_index: &SafeHeaderIndex,
    logger: &Logger,
    ibd: bool,
) -> Result<(), NodeError> {
    let block = match BlockMessage::from_bytes(&msg_bytes) {
        Ok(block_msg) => block_msg.block,
        Err(_) => return Err(NodeError::ErrorReceivingBroadcastedBlock),
    };

    let mut blockchain = safe_blockchain
        .lock()
        .map_err(|_| NodeError::ErrorSharingReference)?;

    let block_header = block.get_header();

    if blockchain.contains_key(&block_header.hash()) {
        return Ok(());
    }

    if !validate_proof_of_work(&block_header) {
        logger.log(String::from("Proof of work failed for a block"));
        return Err(NodeError::ErrorValidatingBlock);
    };
    if !validate_block_proof_of_inclusion(&block) {
        logger.log(String::from("Proof of inclusion failed for a block"));
        return Err(NodeError::ErrorValidatingBlock);
    };
    
    let mut pending_tx = safe_pending_tx
        .lock()
        .map_err(|_| NodeError::ErrorSharingReference)?;
    for tx in block.get_transactions() {
        if pending_tx.remove(&tx.hash()).is_some() {
            logger.log(String::from("Transaccion sacada de pending"));
        }
    }

    if !ibd {
        insert_new_headers(vec![block_header], safe_headers, safe_headers_index)?;
    }

    blockchain.insert(block.header_hash(), block);

    Ok(())
}

///Handles the inv message by asking for the blocks that are not in the blockchain.
///If the block is already in the blockchain, it is not saved.
pub fn handle_inv_message(
    stream: &mut TcpStream,
    msg_bytes: Vec<u8>,
    safe_blockchain: &SafeBlockChain,
    safe_pending_tx: &SafePendingTx,
) -> Result<(), NodeError> {
    let inv_msg = match InvMessage::from_bytes(&msg_bytes) {
        Ok(msg) => msg,
        Err(_) => return Err(NodeError::ErrorRecevingBroadcastedInventory),
    };

    let block_hashes = inv_msg.get_block_hashes();
    let transaction_hashes = inv_msg.get_transaction_hashes();

    let mut request_block_hashes = Vec::new();
    let mut request_transaction_hashes = Vec::new();

    match safe_blockchain.lock() {
        Ok(blockchain) => {
            for hash in block_hashes {
                if !blockchain.contains_key(&hash) {
                    request_block_hashes.push(hash);
                }
            }
        }
        Err(_) => return Err(NodeError::ErrorSharingReference),
    };

    match safe_pending_tx.lock() {
        Ok(pending_tx) => {
            for hash in transaction_hashes {
                if !pending_tx.contains_key(&hash) {
                    request_transaction_hashes.push(hash);
                }
            }
        }
        Err(_) => return Err(NodeError::ErrorSharingReference),
    };
    if !request_block_hashes.is_empty() {
        send_get_data_message_for_blocks(request_block_hashes, stream)
            .map_err(|_| NodeError::ErrorDownloadingBlockBundle)?;
    }
    if !request_transaction_hashes.is_empty() {
        send_get_data_message_for_transactions(request_transaction_hashes, stream)?;
    }
    Ok(())
}

///Handles the ping message by sending a pong message.
pub fn handle_ping_message(
    stream: &mut TcpStream,
    header_message: &HeaderMessage,
    nonce: Vec<u8>,
) -> Result<(), NodeError> {
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

///Handles the block message by validating the proof of work and the proof of inclusion and the saving it.
/// If the block is already in the blockchain, it is not saved.
pub fn handle_tx_message(
    msg_bytes: Vec<u8>,
    safe_pending_tx: &SafePendingTx,
) -> Result<(), NodeError> {
    let tx = match TxMessage::from_bytes(&msg_bytes) {
        Ok(tx_msg) => tx_msg.tx,
        Err(_) => return Err(NodeError::ErrorReceivingTx),
    };
    let mut pending_tx = safe_pending_tx
        .lock()
        .map_err(|_| NodeError::ErrorSharingReference)?;
    pending_tx.insert(tx.hash(), tx);
    Ok(())
}

pub fn handle_get_headers_message(stream: &mut TcpStream, msg_bytes: Vec<u8>, safe_headers: &SafeVecHeader, safe_headers_index: &SafeHeaderIndex, logger: &Logger) -> Result<(), NodeError> {
    let get_headers_msg = match GetBlockHeadersMessage::from_bytes(&msg_bytes) {
        Ok(get_headers_msg) => get_headers_msg,
        Err(_) => return Err(NodeError::ErrorReceivingGetHeaders),
    };
    
    let starting_header_position = match get_starting_header_position(&get_headers_msg, safe_headers_index){
        Ok(header_position) => {
            match header_position {
                Some(header_position) => header_position + 1,
                None => 0,
            }
        },
        Err(NodeError::ErrorFindingBlock) => return Ok(()),
        Err(error) => return Err(error),
    };
    
    let headers_to_send = get_headers_to_send(safe_headers, starting_header_position, &get_headers_msg.stopping_hash)?;
    
    logger.log(format!("Sending {} headers", headers_to_send.len()));
    
    BlockHeadersMessage::new(headers_to_send).send_to(stream).map_err(|_| NodeError::ErrorSendingHeadersMsg)?;
    
    logger.log(format!("Sent headers message"));

    Ok(())
}

fn get_headers_to_send(safe_headers: &SafeVecHeader, starting_header_position: usize, stopping_hash: &[u8;32]) -> Result<Vec<BlockHeader>, NodeError>  {
    let mut headers_to_send: Vec<BlockHeader> = Vec::new();
    
    match safe_headers.lock(){
        Ok(headers) => {
            let mut header_iter = headers.iter().skip(starting_header_position);

            while let Some(header) = header_iter.next(){
                headers_to_send.push(header.clone());
                if (headers_to_send.len() >= MAX_QUANTITY_FOR_GET_HEADERS) || (header.hash() == *stopping_hash){
                    break;
                }
            }
        },
        Err(_) => return Err(NodeError::ErrorSharingReference),
    };

    Ok(headers_to_send)
}

fn get_starting_header_position(get_headers_msg: &GetBlockHeadersMessage, safe_headers_index: &SafeHeaderIndex) -> Result<Option<usize>, NodeError>{
    match safe_headers_index.lock() {
        Ok(header_index) => {
            for header_hash in &get_headers_msg.block_header_hashes{
                if let Some(starting_header_position) = header_index.get(header_hash){
                    return Ok(Some(*starting_header_position));
                }
                if *header_hash == HASHEDGENESISBLOCK{
                    return Ok(None);
                };
            }
            Err(NodeError::ErrorFindingBlock)
        },
        Err(_) => Err(NodeError::ErrorSharingReference),
    }
}

/// Sends a getdata message to the stream, requesting the blocks with the specified hashes.
/// Returns an error if it was not possible to send the message.
fn send_get_data_message_for_transactions(
    hashes: Vec<[u8; 32]>,
    stream: &mut TcpStream,
) -> Result<(), NodeError> {
    let get_data_message = GetDataMessage::create_message_inventory_transaction_type(hashes);

    match get_data_message.send_to(stream) {
        Ok(_) => Ok(()),
        Err(_) => Err(NodeError::ErrorGettingTx),
    }
}


