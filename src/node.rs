use std::{net::{SocketAddr, ToSocketAddrs, TcpStream}, io::Read};
use proyecto::messages::*;

// use messages::VersionMessage;

const DNS_PORT: u16 = 53; //DNS PORT
const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 1001;
const MESAGE_HEADER_SIZE: usize = 24;

/// Struct that represents the errors that can occur in the Node
enum NodeError{
    ErrorConnectingToPeer,
    ErrorCreatingVersionMessageInHandshake,
    ErrorSendingVersionMessageInHandshake,
    ErrorReadingVersionMessageInHandshake,
}

/// Struct that represents the bitcoin node
struct Node {
    version: i32,
    sender_address: SocketAddr,
}

impl Node {
    pub fn new() -> Node {
        Node { 
            version: 70015,
            sender_address: SocketAddr::from((LOCAL_HOST, LOCAL_PORT)),
        }
    }
    /// Receives a dns address as a String and returns a Vector that contains all the addresses
    /// returned by the dns. If an error occured (for example, the dns address is not valid), it
    /// returns an empty Vector.
    /// The socket address requires a dns and a DNS_PORT, which is set to 53 by default
    fn peer_discovery(&self, dns: String) -> Vec<SocketAddr> {
        let mut socket_address_vector = Vec::new();
        match (dns, DNS_PORT).to_socket_addrs() {
            Ok(address_iter) => {
                for address in address_iter {
                    socket_address_vector.push(address);
                }
            }
            Err(_) => {}
        }
        socket_address_vector
    }
     
    // pub fn handshake(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError>{
    //     let mut tcp_stream = match TcpStream::connect(&receiving_addrs){
    //         Ok(stream) => stream,
    //         Err(_) => return Err(NodeError::ErrorConnectingToPeer),
    //     };
    //     let vm = match VersionMessage::new(self.version, receiving_addrs, self.sender_address){
    //         Ok(version_message) => version_message,
    //         Err(_) => return Err(NodeError::ErrorCreatingVersionMessageInHandshake),
    //     };
    //     match vm.send_to(&mut tcp_stream){
    //         Ok(_) => {},
    //         Err(_) => return Err(NodeError::ErrorSendingVersionMessageInHandshake),
    //     }
    //     let mut header_bytes = [0;MESAGE_HEADER_SIZE];
    //     match tcp_stream.read(&mut header_bytes) {
    //         Ok(amount) => {},
    //         Err(_) => return Err(NodeError::ErrorReadingVersionMessageInHandshake),
    //     }
    //     HeaderMessage::from(&mut header_bytes);
        
        //armar header
        //leer tanto como indique header
        //armar version message

        //VersionMessage::from(leido);
          //       -recibir Vms
        //Mandar y recibir ACK
    //}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_peer_discovery_fails_when_receiving_invalid_dns() {
        let node = Node::new();
        let address_vector = node.peer_discovery(String::from("does_not_exist"));
        assert!(address_vector.is_empty());
    }

    #[test]
    fn test_2_peer_discovery_returns_ip_vector_when_receiving_valid_dns() {
        let node = Node::new();
        let address_vector =
            node.peer_discovery(String::from("testnet-seed.bitcoin.jonasschnelli.ch"));
        assert!(!address_vector.is_empty());
    }
}
