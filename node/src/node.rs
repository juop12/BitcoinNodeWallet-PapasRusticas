pub mod peer_comunication;
pub mod data_handler;
pub mod handle_messages;
pub mod handshake;
pub mod initial_block_download;
pub mod utxo_set;
pub mod wallet_communication;


use self::{
    peer_comunication::*,
    data_handler::NodeDataHandler,
    handle_messages::*, 
    message_receiver::MessageReceiver,
};
use crate::{
    blocks::{
        transaction::TxOut,
        blockchain::*,
        proof::*, 
        Transaction, 
        Outpoint,
    },
    messages::*,
    utils::btc_errors::NodeError,
    utils::{config::*, log::*},
};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex, MutexGuard},
};


const MESSAGE_HEADER_SIZE: usize = 24;
const DNS_ADDRESS: &str = "seed.testnet.bitcoin.sprovoost.nl";


pub type SafeBlockChain = Arc<Mutex<HashMap<[u8; 32], Block>>>;
pub type SafeVecHeader = Arc<Mutex<Vec<BlockHeader>>>;
pub type SafePendingTx = Arc<Mutex<HashMap<[u8;32],Transaction>>>;

/// Struct that represents the bitcoin node
pub struct Node {
    version: i32,
    sender_address: SocketAddr,
    pub tcp_streams: Vec<TcpStream>,
    data_handler: NodeDataHandler,
    block_headers: SafeVecHeader,
    starting_block_time: u32,
    blockchain: SafeBlockChain,
    utxo_set: HashMap<Outpoint, TxOut>,
    pub message_receiver: Option<MessageReceiver>,
    pub balance: i64,
    pub pending_tx: SafePendingTx,
    last_proccesed_block: usize,
    wallet_pk_hash: [u8;20],

    headers_in_disk: usize,
    pub logger: Logger,
}

impl Node {
    /// It creates and returns a Node with the default values
    fn _new(
        version: i32,
        local_host: [u8; 4],
        local_port: u16,
        logger: Logger,
        data_handler: NodeDataHandler,
        starting_block_time: u32,
    ) -> Node {
        Node {
            version,
            sender_address: SocketAddr::from((local_host, local_port)),
            tcp_streams: Vec::new(),
            block_headers: Arc::new(Mutex::from(Vec::new())),
            starting_block_time,
            blockchain: Arc::new(Mutex::from(HashMap::new())),
            utxo_set: HashMap::new(),
            message_receiver: None,
            data_handler,
            pending_tx: Arc::new(Mutex::from(HashMap::new())),
            balance: 0,
            last_proccesed_block: 0,
            wallet_pk_hash: [0;20],
            headers_in_disk: 0,
            logger,
        }
    }

    /// Node constructor, it creates a new node and performs the handshake with the sockets obtained
    /// by doing peer_discovery. If the handshake is successful, it adds the socket to the
    /// tcp_streams vector. Returns the node
    pub fn new(config: Config) -> Result<Node, NodeError> {
        
        let logger = Logger::from_path(config.log_path.as_str()).map_err(|_| NodeError::ErrorCreatingNode)?;

        let data_handler = NodeDataHandler::new(&config.headers_path, &config.blocks_path).map_err(|_| NodeError::ErrorCreatingNode)?;

        let mut node = Node::_new(
            config.version,
            config.local_host,
            config.local_port,
            logger,
            data_handler,
            config.begin_time,
        );

        let mut address_vector = node.peer_discovery(DNS_ADDRESS, config.dns_port, config.ipv6_enabled);
        address_vector.reverse(); // Generally the first nodes are slow, so we reverse the vector to connect to the fastest nodes first

        for addr in address_vector {
            match node.handshake(addr) {
                Ok(tcp_stream) => node.tcp_streams.push(tcp_stream),
                Err(error) => node.logger.log_error(&error),
            }
        }

        node.logger.log(format!(
            "Amount of peers conected = {}",
            node.tcp_streams.len()
        ));

        if node.tcp_streams.is_empty() {
            Err(NodeError::ErrorCreatingNode)
        } else {
            Ok(node)
        }
    }

    /// Receives a dns address as a String and returns a Vector that contains all the addresses
    /// returned by the dns. If an error occured (for example, the dns address is not valid), it
    /// returns an empty Vector.
    /// The socket address requires a dns and a DNS_PORT, which is set to 18333 because it is
    /// the port used by the bitcoin core testnet.
    fn peer_discovery(&self, dns: &str, dns_port: u16, ipv6_enabled: bool) -> Vec<SocketAddr> {
        let mut socket_address_vector = Vec::new();

        if let Ok(address_iter) = (dns, dns_port).to_socket_addrs() {
            for address in address_iter {
                if address.is_ipv4() || ipv6_enabled {
                    socket_address_vector.push(address);
                }
            }
        }
        socket_address_vector
    }

    /// Returns a MutexGuard to the blockchain HashMap. 
    pub fn get_blockchain(&self) -> Result<MutexGuard<HashMap<[u8; 32], Block>>, NodeError>{
        self.blockchain.lock().map_err(|_| NodeError::ErrorSharingReference)
    }

    /// Returns a MutexGuard to the blockchain HashMap. 
    pub fn get_block_headers(&self) -> Result<MutexGuard<Vec<BlockHeader>>, NodeError>{
        self.block_headers.lock().map_err(|_| NodeError::ErrorSharingReference)
    }

    /// Returns a MutexGuard to the pending tx HashMap. 
    pub fn get_pending_tx(&self) -> Result<MutexGuard<HashMap<[u8;32],Transaction>>, NodeError>{
        self.pending_tx.lock().map_err(|_| NodeError::ErrorSharingReference)
    }

    /// Creates a threadpool responsable for receiving messages in different threads.
    pub fn start_receiving_messages(&mut self){
        self.message_receiver = Some(MessageReceiver::new(&self.tcp_streams, &self.blockchain, &self.block_headers, &self.pending_tx ,&self.logger));
    }

    /// Actual receive_message wrapper. Encapsulates node's parameteres.
    fn receive_message(&mut self, stream_index: usize, ibd: bool) -> Result<String, NodeError>{
        let stream = &mut self.tcp_streams[stream_index];
        receive_message(stream, &self.block_headers, &self.blockchain, &self.pending_tx, &self.logger, ibd)
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        // Finishing every thread gracefully.
        if let Some(message_receiver) = self.message_receiver.take(){
            if let Err(error) = message_receiver.finish_receiving(){
                self.logger.log_error(&error);
            }
        }
        
        //Saving data.
        self.logger.log(format!("Saving received data"));

        if self.store_headers_in_disk().is_err(){
            return self.logger.log_error(&NodeError::ErrorSavingDataToDisk);
        };

        if self.store_blocks_in_disk().is_err(){
            return self.logger.log_error(&NodeError::ErrorSavingDataToDisk);
        };

        self.logger.log(format!("Finished storing data"));
    }
}

/// Reads from the stream MESAGE_HEADER_SIZE bytes and returns a HeaderMessage interpreting those bytes acording to bitcoin protocol.
/// On error returns ErrorReceivingMessage
pub fn receive_message_header<T: Read + Write>(stream: &mut T) -> Result<HeaderMessage, NodeError> {
    let mut header_bytes = [0; MESSAGE_HEADER_SIZE];

    if stream.read_exact(&mut header_bytes).is_err() {
        return Err(NodeError::ErrorReceivingMessageHeader);
    };

    match HeaderMessage::from_bytes(&header_bytes) {
        Ok(header_message) => Ok(header_message),
        Err(_) => Err(NodeError::ErrorReceivingMessageHeader),
    }
}

/// Generic receive message function, receives a header and its payload, and calls the corresponding handler. Returns the command name in the received header
pub fn receive_message(stream: &mut TcpStream, block_headers: &SafeVecHeader, blockchain: &SafeBlockChain, pending_tx: &SafePendingTx, logger: &Logger, ibd: bool) -> Result<String, NodeError> {
    let block_headers_msg_h = receive_message_header(stream)?;

    logger.log(format!("Received message: {}", block_headers_msg_h.get_command_name()));

    let mut msg_bytes = vec![0; block_headers_msg_h.get_payload_size() as usize];
    
    if stream.read_exact(&mut msg_bytes).is_err() {
        return Err(NodeError::ErrorReceivingMessage);
    }

    match block_headers_msg_h.get_command_name().as_str() {
        "ping\0\0\0\0\0\0\0\0" => {
            handle_ping_message(stream, &block_headers_msg_h, msg_bytes)?;
        }
        "inv\0\0\0\0\0\0\0\0\0" => {
            if !ibd {
                handle_inv_message(stream, msg_bytes, blockchain, pending_tx)?;
            }
        }
        "block\0\0\0\0\0\0\0" => handle_block_message(msg_bytes, block_headers, blockchain, pending_tx, logger, ibd)?,
        "headers\0\0\0\0\0" => handle_block_headers_message(msg_bytes, block_headers)?,
        "tx\0\0\0\0\0\0\0\0\0\0" => {
            if !ibd{
                handle_tx_message(msg_bytes, pending_tx)?;
            }
        }
        _ => {},
    };

    Ok(block_headers_msg_h.get_command_name())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock_tcp_stream::*;


    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001;
    const DNS_PORT: u16 = 18333;
    const VERSION: i32 = 70015;
    const STARTING_BLOCK_TIME: u32 = 1681084800;
    const LOG_FILE_PATH: &str = "tests_txt/test_log.txt";
    const HEADERS_FILE_PATH: &str = "tests_txt/headers.bin";
    const BLOCKS_FILE_PATH: &str = "tests_txt/blocks.bin";


    #[test]
    fn peer_discovery_test_1_fails_when_receiving_invalid_dns_address() {
        let logger = Logger::from_path(LOG_FILE_PATH).unwrap();
        let data_handler = NodeDataHandler::new(HEADERS_FILE_PATH, BLOCKS_FILE_PATH).unwrap();
        let node = Node::_new(
            VERSION,
            LOCAL_HOST,
            LOCAL_PORT,
            logger,
            data_handler,
            STARTING_BLOCK_TIME,
        );
        let address_vector = node.peer_discovery("does_not_exist", DNS_PORT, false);

        assert!(address_vector.is_empty());
    }

    #[test]
    fn peer_discovery_test_2_returns_ip_vector_when_receiving_valid_dns() {
        let logger = Logger::from_path(LOG_FILE_PATH).unwrap();
        let data_handler = NodeDataHandler::new(HEADERS_FILE_PATH, BLOCKS_FILE_PATH).unwrap();
        let node = Node::_new(
            VERSION,
            LOCAL_HOST,
            LOCAL_PORT,
            logger,
            data_handler,
            STARTING_BLOCK_TIME,
        );
        let address_vector = node.peer_discovery(DNS_ADDRESS, DNS_PORT, false);

        assert!(!address_vector.is_empty());
    }

    #[test]
    fn node_test_1_receive_header_message() -> Result<(), NodeError> {
        let mut stream = MockTcpStream::new();

        let expected_hm =
            HeaderMessage::new("test message", &Vec::from("test".as_bytes())).unwrap();
        stream.read_buffer = expected_hm.to_bytes();

        let received_hm = receive_message_header(&mut stream)?;

        assert_eq!(received_hm, expected_hm);
        Ok(())
    }
}
