pub mod initial_block_download;
pub mod handshake;
pub mod block_downloader;
pub mod data_handler;

use crate::blockchain::*;
use crate::messages::*;
use crate::config::*;
use crate::log::*;
use std::{
    io::{Read, Write},
    net::{SocketAddr, ToSocketAddrs, TcpStream},
};


const MESSAGE_HEADER_SIZE: usize = 24;
const DNS_ADDRESS: &str = "seed.testnet.bitcoin.sprovoost.nl";


/// Struct that represents the errors that can occur in the Node
#[derive(Debug)]
pub enum NodeError {
    ErrorConnectingToPeer,
    ErrorSendingMessageInHandshake,
    ErrorReceivingMessageInHandshake,
    ErrorReceivedUnknownMessage,
    ErrorInterpretingMessageCommandName,
    ErrorUnknownCommandName,
    ErrorSendingMessageInIBD,
    ErrorIteratingStreams,
    ErrorReceivingHeadersMessageInIBD,
    ErrorReceivingMessageHeader,
    ErrorReceivingHeadersMessageHeaderInIBD,
    ErrorCreatingBlockDownloader,
    ErrorDownloadingBlockBundle,
    ErrorCreatingNode,
    ErrorSavingDataToDisk,
    ErrorLoadingDataFromDisk,
}

/* 
impl BTCError for NodeError{

    fn decode(&self) -> String{
        let message = match self {
            NodeError::ErrorConnectingToPeer => "",
            NodeError::ErrorSendingMessageInHandshake => "",
            NodeError::ErrorReceivingMessageInHandshake => "",
            NodeError::ErrorReceivedUnknownMessage => "",
            NodeError::ErrorInterpretingMessageCommandName => "",
            NodeError::ErrorUnknownCommandName => "",
            NodeError::ErrorSendingMessageInIBD => "",
            NodeError::ErrorIteratingStreams => "",
            NodeError::ErrorReceivingHeadersMessageInIBD => "",
            NodeError::ErrorReceivingMessageHeader => "", 
            NodeError::ErrorReceivingHeadersMessageHeaderInIBD => "",
        };

        message.to_string()
    }
} 
*/


/// Struct that represents the bitcoin node
pub struct Node {
    version: i32,
    sender_address: SocketAddr,
    tcp_streams: Vec<TcpStream>,
    block_headers: Vec<BlockHeader>,
    blockchain: Vec<Block>,
    logger: Logger,
}

impl Node {

    /// It creates and returns a Node with the default values
    fn _new(version: i32, local_host: [u8; 4], local_port: u16, logger: Logger) -> Node {
        Node {
            version,
            sender_address: SocketAddr::from((local_host, local_port)),
            tcp_streams: Vec::new(),
            block_headers: Vec::new(),
            blockchain: Vec::new(),
            logger,
        }
    }

    /// Node constructor, it creates a new node and performs the handshake with the sockets obtained
    /// by doing peer_discovery. If the handshake is successful, it adds the socket to the
    /// tcp_streams vector. Returns the node
    pub fn new(config: Config) -> Result<Node, NodeError> {
        let logger = match Logger::from_path(config.log_path.as_str()){
            Ok(logger) => logger,
            Err(_) => return Err(NodeError::ErrorCreatingNode),
        };
        let mut node = Node::_new(config.version, config.local_host, config.local_port, logger);
        let address_vector = node.peer_discovery(DNS_ADDRESS, config.dns_port);
        
        for addr in address_vector {
            if let Ok(tcp_stream) = node.handshake(addr) {
                node.tcp_streams.push(tcp_stream);
            }
        }

        Ok(node)
    }

    /// Receives a dns address as a String and returns a Vector that contains all the addresses
    /// returned by the dns. If an error occured (for example, the dns address is not valid), it
    /// returns an empty Vector.
    /// The socket address requires a dns and a DNS_PORT, which is set to 53 by default
    fn peer_discovery(&self, dns: &str, dns_port: u16) -> Vec<SocketAddr> {
        let mut socket_address_vector = Vec::new();

        if let Ok(address_iter) = (dns, dns_port).to_socket_addrs() {
            for address in address_iter {
                socket_address_vector.push(address);
            }
        }
        socket_address_vector
    }

    /// Returns a reference to the tcp_streams vector
    pub fn get_tcp_streams(&self) -> &Vec<TcpStream> {
        &self.tcp_streams
    }

    pub fn get_blockchain(&self) -> &Vec<Block>{
        &self.blockchain
    }
    /*
            fn handle_received_verack_message(&self, message_bytes: Vec<u8>)-> Result<(), NodeError>{
                let vm = match VersionMessage::from_bytes(message_bytes) {
                    Ok(version_message) => version_message,
                    Err(_) => return Err(NodeError::ErrorReceiving)
                }
            }

            fn handle_received_version_message(&self, message_bytes: Vec<u8>)-> Result<(), NodeError> {

            }

            fn receive_message(&self, mut stream: TcpStream)-> Result<(), NodeError>{
                let hm = self.receive_header_message(&stream)?;

                let mut message_bytes = Vec::with_capacity(hm.get_payload_size() as usize);
                match stream.read_exact(&mut message_bytes) {
                    Ok(_) => {}
                    Err(_) => return Err(NodeError::ErrorReceivingMessageInHandshake),
                };

                let command_name = match hm.get_command_name(){
                    Ok(string) => string.as_str(),
                    Err(_) => return Err(NodeError::ErrorUnknownCommandName),
                };
                //handle
                match command_name {
                    "version\0\0\0\0\0" => self.handle_received_version_message(message_bytes),
                    "verack\0\0\0\0\0\0" => self.handle_received_verack_message(message_bytes),
                    _ => return Err(NodeError::ErrorUnknownCommandName),
                };
                Ok(())
            }
    */
}

///Reads from the stream MESAGE_HEADER_SIZE bytes and returns a HeaderMessage interpreting those bytes acording to bitcoin protocol.
/// On error returns ErrorReceivingMessage
pub fn receive_message_header<T: Read + Write>(stream: &mut T,) -> Result<HeaderMessage, NodeError> {
    let mut header_bytes = [0; MESSAGE_HEADER_SIZE];
    if let Err(error) = stream.read_exact(&mut header_bytes){
        return Err(NodeError::ErrorReceivingMessageHeader);
    };
    match HeaderMessage::from_bytes(&mut header_bytes) {
        Ok(header_message) => Ok(header_message),
        Err(_) => Err(NodeError::ErrorReceivingMessageHeader),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_tcp_stream::*;


    const VERSION: i32 = 70015;
    const DNS_PORT: u16 = 18333;
    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001; 


    #[test]
    fn peer_discovery_test_1_fails_when_receiving_invalid_dns_address() {
        let logger = Logger::from_path("test_log.txt").unwrap();
        let node = Node::_new(VERSION, LOCAL_HOST, LOCAL_PORT, logger);
        let address_vector = node.peer_discovery("does_not_exist", DNS_PORT);

        assert!(address_vector.is_empty());
    }

    #[test]
    fn peer_discovery_test_2_returns_ip_vector_when_receiving_valid_dns() {
        let logger = Logger::from_path("test_log.txt").unwrap();
        let node = Node::_new(VERSION, LOCAL_HOST, LOCAL_PORT, logger);
        let address_vector = node.peer_discovery(DNS_ADDRESS, DNS_PORT);

        assert!(!address_vector.is_empty());
    }

    #[test]
    fn node_test_1_receive_header_message() -> Result<(), NodeError> {
        let mut stream = MockTcpStream::new();

        let logger = Logger::from_path("test_log.txt").unwrap();
        let node = Node::_new(VERSION, LOCAL_HOST, LOCAL_PORT, logger);

        let expected_hm =
            HeaderMessage::new("test message", &Vec::from("test".as_bytes())).unwrap();
        stream.read_buffer = expected_hm.to_bytes();

        let received_hm = receive_message_header(&mut stream)?;

        assert_eq!(received_hm, expected_hm);
        Ok(())
    }
}