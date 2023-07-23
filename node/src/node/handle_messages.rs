use crate::node::*;
use crate::node::initial_block_download::HASHEDGENESISBLOCK;
use crate::node::get_block_headers_message::MAX_QUANTITY_FOR_GET_HEADERS;

use super::peer_comunication::block_downloader::send_get_data_message_for_blocks;

pub fn handle_message(
    message: Message,
    stream: &mut TcpStream,
    block_headers: &SafeVecHeader,
    blockchain: &SafeBlockChain,
    pending_tx: &SafePendingTx,
    headers_index: &SafeHeaderIndex,
    logger: &Logger,
    ibd: bool)->Result<(), NodeError>{
        match message{
            Message::BlockHeaders(msg) => if ibd {
                handle_block_headers_message(msg, block_headers, headers_index)?;
            },
            Message::Block(msg) => handle_block_message(msg, block_headers, blockchain, pending_tx, headers_index, logger, ibd)?,
            Message::GetBlockHeaders(msg) => if !ibd{
                handle_get_headers_message(stream, msg, block_headers, headers_index, logger)?;
            },
            Message::GetData(msg) => if !ibd{
                handle_get_data(stream, msg, blockchain)?;
            },
            Message::Header(_) => return Err(NodeError::DoubleHeader),
            Message::Inv(msg) => if !ibd{
                handle_inv_message(stream, msg, blockchain, pending_tx)?;
            },
            Message::Tx(msg) => handle_tx_message(msg, pending_tx)?,
            Message::Ping(msg) => handle_ping_message(stream, msg)?,
            _ => {},
        };
        Ok(())
    }

///Handles the headers message by hashing the last received header and asking for more headers.
pub fn handle_block_headers_message(
    block_headers_msg : BlockHeadersMessage,
    safe_block_headers: &SafeVecHeader,
    safe_headers_index: &SafeHeaderIndex
) -> Result<(), NodeError> {
    let received_block_headers = block_headers_msg.headers;

    insert_new_headers(received_block_headers, &safe_block_headers, &safe_headers_index)?;

    Ok(())
}

/// Handles the block message by validating the proof of work and the proof of inclusion and the saving it.
/// If the block is already in the blockchain, it is not saved.
pub fn handle_block_message(
    block_msg: BlockMessage,
    safe_headers: &SafeVecHeader,
    safe_blockchain: &SafeBlockChain,
    safe_pending_tx: &SafePendingTx,
    safe_headers_index: &SafeHeaderIndex,
    logger: &Logger,
    ibd: bool,
) -> Result<(), NodeError> {
    let block = block_msg.block;

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
    
    match safe_pending_tx.lock(){
        Ok(mut pending_tx) => {
            for tx in block.get_transactions() {
                if pending_tx.remove(&tx.hash()).is_some() {
                    logger.log(String::from("Transaccion sacada de pending"));
                }
            }
        },
        Err(_) => return Err(NodeError::ErrorSharingReference),
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
    inv_msg: InvMessage,
    safe_blockchain: &SafeBlockChain,
    safe_pending_tx: &SafePendingTx,
) -> Result<(), NodeError> {
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
    ping_msg: PingMessage,
) -> Result<(), NodeError> {
    ping_msg.reply_pong(stream).map_err(|error| NodeError::ErrorMessage(error))
}

///Handles the block message by validating the proof of work and the proof of inclusion and the saving it.
/// If the block is already in the blockchain, it is not saved.
pub fn handle_tx_message(
    tx_msg: TxMessage,
    safe_pending_tx: &SafePendingTx,
) -> Result<(), NodeError> {
    let tx = tx_msg.tx;
    let mut pending_tx = safe_pending_tx
        .lock()
        .map_err(|_| NodeError::ErrorSharingReference)?;
    pending_tx.insert(tx.hash(), tx);
    Ok(())
}

/// Handles the get_headers_message answearing with a Header message cointaining the headers starting from the
/// latest block in the blockchain that is shared between the local blockchain and the get_headers_message, and 
/// stopping when either of the following conditions is met:
/// -The stopping_hash is found
/// -The end of the blockchain is reached
/// -The len of the vector reaches MAX_QUANTITY_FOR_GET_HEADERS 
pub fn handle_get_headers_message(stream: &mut TcpStream, get_headers_msg: GetBlockHeadersMessage, safe_headers: &SafeVecHeader, safe_headers_index: &SafeHeaderIndex, logger: &Logger) -> Result<(), NodeError> {
    
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
    
    BlockHeadersMessage::new(headers_to_send).send_to(stream).map_err(|_| NodeError::ErrorMessage(MessageError::ErrorSendingBlockHeadersMessage))?;
    
    logger.log(format!("Sent headers message"));

    Ok(())
}

/// Returns a vector of Header, filling it from starting_header_position, until either of the following conditions is met
/// -The stopping_hash is found
/// -The end of the blockchain is reached
/// -The len of the vector reaches MAX_QUANTITY_FOR_GET_HEADERS 
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

/// Gets the latest block in the blockchain that is shared between the local blockchain and the get_headers_message
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

/// Handles get data message. If it receives block hashes it looks for them in the block chain and responds 
/// with Block messages, each block hash not found in the blocks is then returned through a NotFoundMessage 
pub fn handle_get_data(stream: &mut TcpStream, get_data_msg: GetDataMessage, safe_blockchain: &SafeBlockChain) -> Result<(), NodeError>{
    let hashes = get_data_msg.get_block_hashes();

    let mut block_messages = Vec::new();
    let mut not_found_blocks = Vec::new();
    match safe_blockchain.lock(){
        Ok(blockchain) => {
            for hash in hashes{
                match blockchain.get(&hash){
                    Some(block) => block_messages.push(BlockMessage::from(block).map_err(|_| NodeError::ErrorMessage(MessageError::ErrorCreatingGetDataMessage))?),
                    None => {
                        not_found_blocks.push(hash);
                    }
                };
            }
        },
        Err(_) => return Err(NodeError::ErrorSharingReference),
    }
    
    for message in block_messages{
        message.send_to(stream).map_err(|_| NodeError::ErrorMessage(MessageError::ErrorSendingBlockMessage))?;
    }
    
    if !not_found_blocks.is_empty(){
        NotFoundMessage::from_block_hashes(not_found_blocks).send_to(stream).map_err(|_| NodeError::ErrorMessage(MessageError::ErrorSendingNotFoundMessage))?;
    }
    Ok(())
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
        Err(_) => Err(NodeError::ErrorMessage(MessageError::ErrorSendingGetDataMessage)),
    }
}


