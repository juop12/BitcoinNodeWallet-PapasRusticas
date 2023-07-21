pub mod data_handler;
pub mod handle_messages;
pub mod handshake;
pub mod initial_block_download;
pub mod peer_comunication;
pub mod utxo_set;
pub mod wallet_communication;

use self::{
    data_handler::NodeDataHandler, handle_messages::*, peer_comunicator::PeerComunicator,
    peer_comunication::*, 
    handshake::*,
};
use crate::{
    blocks::{blockchain::*, proof::*, transaction::TxOut, Outpoint, Transaction},
    messages::{*, message_trait::MessageError},
    utils::{btc_errors::NodeError,config::*, log::*, UIResponse, LoadingScreenInfo},
};
use std::{
    collections::HashMap,
    io::{Read, Write, ErrorKind::WouldBlock},
    net::{SocketAddr, TcpStream, ToSocketAddrs, TcpListener},
    sync::{Arc, Mutex, MutexGuard},
    thread::sleep, 
    time::Duration,
};
use glib::Sender as GlibSender;

const MESSAGE_HEADER_SIZE: usize = 24;

pub type SafeBlockChain = Arc<Mutex<HashMap<[u8; 32], Block>>>;
pub type SafeVecHeader = Arc<Mutex<Vec<BlockHeader>>>;
pub type SafePendingTx = Arc<Mutex<HashMap<[u8; 32], Transaction>>>;
pub type SafeHeaderIndex = Arc<Mutex<HashMap<[u8; 32], usize>>>;

/// Struct that represents the bitcoin node
pub struct Node {
    version: i32,
    address: SocketAddr,
    pub initial_peers: Vec<TcpStream>,
    data_handler: NodeDataHandler,
    block_headers: SafeVecHeader,
    headers_index: SafeHeaderIndex,
    starting_block_time: u32,
    blockchain: SafeBlockChain,
    utxo_set: HashMap<Outpoint, TxOut>,
    pub peer_comunicator: Option<PeerComunicator>,
    pub balance: i64,
    pub pending_tx: SafePendingTx,
    last_proccesed_block: usize,
    wallet_pk_hash: [u8; 20],
    headers_in_disk: usize,
    pub logger: Logger,
    pub sender_to_ui: GlibSender<UIResponse>,
}

impl Node {
    /// It creates and returns a Node with the default values
    fn _new(
        version: i32,
        local_address: ([u8; 4], u16),
        logger: Logger,
        data_handler: NodeDataHandler,
        starting_block_time: u32,
        sender_to_ui: GlibSender<UIResponse>
    ) -> Node {
        Node {
            version,
            address: SocketAddr::from(local_address),
            initial_peers: Vec::new(),
            block_headers: Arc::new(Mutex::from(Vec::new())),
            headers_index: Arc::new(Mutex::from(HashMap::new())),
            starting_block_time,
            blockchain: Arc::new(Mutex::from(HashMap::new())),
            utxo_set: HashMap::new(),
            peer_comunicator: None,
            data_handler,
            pending_tx: Arc::new(Mutex::from(HashMap::new())),
            balance: 0,
            last_proccesed_block: 0,
            wallet_pk_hash: [0; 20],
            headers_in_disk: 0,
            logger,
            sender_to_ui,
        }
    }

    /// Node constructor, it creates a new node and performs the handshake with the sockets obtained
    /// by doing peer_discovery. If the handshake is successful, it adds the socket to the
    /// tcp_streams vector. Returns the node
    pub fn new(config: Config, sender_to_ui: GlibSender<UIResponse>) -> Result<Node, NodeError> {
        let logger = Logger::from_path(config.log_path.as_str())
            .map_err(|_| NodeError::ErrorCreatingNode)?;

        let data_handler = NodeDataHandler::new(&config.headers_path, &config.blocks_path)
            .map_err(|_| NodeError::ErrorCreatingNode)?;

        let mut node = Node::_new(
            config.version,
            config.local_address,
            logger,
            data_handler,
            config.begin_time,
            sender_to_ui,
        );

        let mut address_vector = node.peer_discovery(config.dns, config.ipv6_enabled);
        address_vector.extend(node.add_external_addresses(config.external_addresses));

        address_vector.reverse(); // Generally the first nodes are slow, so we reverse the vector to connect to the fastest nodes first

        for addr in address_vector {
            match outgoing_handshake(node.version, addr, node.address, &node.logger) {
                Ok(tcp_stream) => {
                    node.initial_peers.push(tcp_stream);
                    let progress = format!(
                        "Amount of peers conected = {}",
                        node.initial_peers.len()
                    );
                    node.log_and_send_to_ui(&progress);
                },
                Err(error) => node.logger.log_error(&error),
            }
        }

        if node.initial_peers.is_empty() {
            Err(NodeError::ErrorCreatingNode)
        } else {
            Ok(node)
        }
    }

    ///-
    pub fn log_and_send_to_ui(&self, message: &str) {
        self.logger.log(message.to_string());
        let ui_message = LoadingScreenInfo::UpdateLabel(message.to_string());
        self.sender_to_ui.send(UIResponse::LoadingScreenUpdate(ui_message)).expect("Error sending message to UI");
    }

    /// Receives a vector of dns address as a (String, u16) each and returns a 
    /// Vector that contains all the addresses returned by the dns. 
    /// If an error occured (for example, the dns address is not valid), it returns an empty Vector.
    /// The socket address requires a dns and a DNS_PORT. 
    /// (DNS_PORT is set to 18333 because it is the port used by the bitcoin core testnet).
    fn peer_discovery(&self, dns_vector: Vec<(String, u16)>, ipv6_enabled: bool) -> Vec<SocketAddr> {
        let mut socket_address_vector = Vec::new();

        for dns in dns_vector{
            if let Ok(address_iter) = dns.to_socket_addrs() {
                for address in address_iter {
                    if address.is_ipv4() || ipv6_enabled {
                        socket_address_vector.push(address);
                    }
                }
            }
        }

        socket_address_vector
    }

    /// Receives a vector of addresses as a ([u8; 4], u16) each and returns a Vector 
    /// that contains all the addresses converted to SocketAddr. 
    /// If an error occured (for example, all addresses are not valid), it returns an empty Vector.
    /// The socket address requires an IP and a PORT. 
    /// (PORT is set to 18333 because it is the port used by the bitcoin core testnet).
    fn add_external_addresses(&self, addresses: Vec<([u8; 4], u16)>) -> Vec<SocketAddr>{
        let mut socket_address_vector = Vec::new();

        for address in addresses{
            let socket_address = SocketAddr::from(address);
            socket_address_vector.push(socket_address); //p Si el usuario quiere poner IPv6 puede. El booleano solo cuenta para las DNS
        }

        socket_address_vector
    }

    /// Returns a MutexGuard to the blockchain HashMap.
    pub fn get_blockchain(&self) -> Result<MutexGuard<HashMap<[u8; 32], Block>>, NodeError> {
        self.blockchain
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }

    /// Returns a MutexGuard to the blockchain HashMap.
    pub fn get_block_headers(&self) -> Result<MutexGuard<Vec<BlockHeader>>, NodeError> {
        self.block_headers
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }

    /// Returns a MutexGuard to the pending tx HashMap.
    pub fn get_pending_tx(&self) -> Result<MutexGuard<HashMap<[u8; 32], Transaction>>, NodeError> {
        self.pending_tx
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }

    pub fn get_header_index(&self) -> Result<MutexGuard<HashMap<[u8; 32], usize>>, NodeError> {
        self.headers_index
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }

    /// Creates a threadpool responsable for receiving messages in different threads.
    pub fn start_receiving_messages(&mut self) {
        self.peer_comunicator = Some(PeerComunicator::new(
            self.version,
            self.address,
            &self.initial_peers,
            &self.blockchain,
            &self.block_headers,
            &self.pending_tx,
            &self.headers_index,
            &self.logger,
        ));
    }

    /// Actual receive_message wrapper. Encapsulates node's parameteres.
    fn receive_message(&mut self, stream_index: usize, ibd: bool) -> Result<String, NodeError> {
        let stream = &mut self.initial_peers[stream_index];
        recieve_and_handle(
            stream,
            &self.block_headers,
            &self.blockchain,
            &self.pending_tx,
            &self.headers_index,
            &self.logger,
            ibd,
        )
    }

}

impl Drop for Node {
    fn drop(&mut self) {
        // Finishing every thread gracefully.
        if let Some(peer_comunicator) = self.peer_comunicator.take() {
            if let Err(error) = peer_comunicator.end_of_communications() {
                self.logger.log_error(&error);
            }
        }

        //Saving data.
        self.logger.log("Saving received data".to_string());

        if self.store_headers_in_disk().is_err() {
            return self.logger.log_error(&NodeError::ErrorSavingDataToDisk);
        };

        if self.store_blocks_in_disk().is_err() {
            return self.logger.log_error(&NodeError::ErrorSavingDataToDisk);
        };

        self.logger.log("Finished storing data".to_string());
    }
}

/// Reads from the stream MESAGE_HEADER_SIZE bytes and returns a HeaderMessage interpreting those bytes acording to bitcoin protocol.
/// On error returns ErrorReceivingMessage
pub fn receive_message_header<T: Read + Write>(stream: &mut T) -> Result<HeaderMessage, NodeError> {
    let mut header_bytes = [0; MESSAGE_HEADER_SIZE];

    stream.read_exact(&mut header_bytes).map_err(|err| {
        if err.kind() == WouldBlock {
            NodeError::ErrorPeerTimeout
        } else {
            NodeError::ErrorReceivingMessageHeader
        }
    })?;

    match HeaderMessage::from_bytes(&header_bytes) {
        Ok(header_message) => Ok(header_message),
        Err(_) => Err(NodeError::ErrorReceivingMessageHeader),
    }
}

fn receive_message(stream: &mut TcpStream, logger: &Logger,)->Result<(Message,String), NodeError>{
    let block_headers_msg_h = receive_message_header(stream)?;

    logger.log(format!(
        "Received message: {}",
        block_headers_msg_h.get_command_name()
    ));

    let mut msg_bytes = vec![0; block_headers_msg_h.get_payload_size() as usize];

    stream.read_exact(&mut msg_bytes).map_err(|err| {
        if err.kind() == WouldBlock {
            NodeError::ErrorPeerTimeout
        } else {
            NodeError::ErrorReceivingMessageHeader
        }
    })?;
    
    let msg = Message::from_bytes(msg_bytes, block_headers_msg_h.get_command_name()).map_err(|msg_error| NodeError::ErrorMessage(msg_error))?;
    Ok((msg, block_headers_msg_h.get_command_name()))
}

pub fn recieve_and_handle(
    stream: &mut TcpStream,
    block_headers: &SafeVecHeader,
    blockchain: &SafeBlockChain,
    pending_tx: &SafePendingTx,
    headers_index: &SafeHeaderIndex,
    logger: &Logger,
    ibd: bool)-> Result<String, NodeError>{

    let (msg, command_name) = receive_message(stream, logger)?;
    handle_message(msg, stream, block_headers, blockchain, pending_tx, headers_index, logger, ibd)?;
    Ok(command_name)
}
/*
/// Generic receive message function, receives a header and its payload, and calls the corresponding handler. Returns the command name in the received header
pub fn receive_message(
    stream: &mut TcpStream,
    block_headers: &SafeVecHeader,
    blockchain: &SafeBlockChain,
    pending_tx: &SafePendingTx,
    headers_index: &SafeHeaderIndex,
    logger: &Logger,
    ibd: bool,
) -> Result<String, NodeError> {
    let block_headers_msg_h = receive_message_header(stream)?;
    
    logger.log(format!(
        "Received message: {}",
        block_headers_msg_h.get_command_name()
    ));

    let mut msg_bytes = vec![0; block_headers_msg_h.get_payload_size() as usize];
    
    stream.read_exact(&mut msg_bytes).map_err(|err| {
        if err.kind() == WouldBlock {
            NodeError::ErrorPeerTimeout
        } else {
            NodeError::ErrorReceivingMessageHeader
        }
    })?;

    match block_headers_msg_h.get_command_name().as_str() {
        "ping\0\0\0\0\0\0\0\0" => {
            handle_ping_message(stream, &block_headers_msg_h, msg_bytes)?;
        }
        "inv\0\0\0\0\0\0\0\0\0" => {
            if !ibd {
                handle_inv_message(stream, msg_bytes, blockchain, pending_tx)?;
            }
        }
        "block\0\0\0\0\0\0\0" => handle_block_message(
            msg_bytes,
            block_headers,
            blockchain,
            pending_tx,
            headers_index,
            logger,
            ibd,
        )?,
        "headers\0\0\0\0\0" => handle_block_headers_message(msg_bytes, block_headers, headers_index)?,
        "getheaders\0\0" => if !ibd{
                handle_get_headers_message(stream, msg_bytes, block_headers, headers_index, logger)?;
            },
        "getdata\0\0\0\0\0" => if !ibd{
                handle_get_data(stream, msg_bytes, blockchain)?;
            },
            "tx\0\0\0\0\0\0\0\0\0\0" => handle_tx_message(msg_bytes, pending_tx)?,
        _ => {}
    };

    Ok(block_headers_msg_h.get_command_name())
}
*/

pub fn insert_new_headers(headers: Vec<BlockHeader>, safe_block_headers: &SafeVecHeader, safe_headers_index: &SafeHeaderIndex) -> Result<(), NodeError>{

    let mut block_headers = safe_block_headers.lock().map_err(|_| NodeError::ErrorSharingReference)?;
    let mut headers_index = safe_headers_index.lock().map_err(|_| NodeError::ErrorSharingReference)?;
    
    for header in headers{
        headers_index.insert(header.hash(), block_headers.len());
        block_headers.push(header);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock_tcp_stream::*;

    const LOCAL_ADDRESS: ([u8; 4], u16) = ([127, 0, 0, 1], 1001);
    const DNS_ADDRESS: &str = "seed.testnet.bitcoin.sprovoost.nl";
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
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let node = Node::_new(
            VERSION,
            LOCAL_ADDRESS,
            logger,
            data_handler,
            STARTING_BLOCK_TIME,
            sx,
        );

        let invalid_dns_address = vec![("does_not_exist".to_string(), DNS_PORT)];

        let address_vector = node.peer_discovery(invalid_dns_address, false);

        assert!(address_vector.is_empty());
    }

    #[test]
    fn peer_discovery_test_2_returns_ip_vector_when_receiving_valid_dns() {
        let logger = Logger::from_path(LOG_FILE_PATH).unwrap();
        let data_handler = NodeDataHandler::new(HEADERS_FILE_PATH, BLOCKS_FILE_PATH).unwrap();
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let node = Node::_new(
            VERSION,
            LOCAL_ADDRESS,
            logger,
            data_handler,
            STARTING_BLOCK_TIME,
            sx,
        );

        let valid_dns_address = vec![(DNS_ADDRESS.to_string(), DNS_PORT)];

        let address_vector = node.peer_discovery(valid_dns_address, false);

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
