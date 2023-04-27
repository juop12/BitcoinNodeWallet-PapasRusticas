use chrono::{Utc};
use rand::prelude::*;
use std::net::{SocketAddr, TcpStream};
use std::io::prelude::*;
use bitcoin_hashes::{sha256d, Hash};

const NODE_NETWORK: u64 = 0x01;
const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 1001;
const START_STRING_TEST_NET : [u8; 4] = [0xd9,0xb4,0xeb,0xf9];

#[derive(Debug)]
pub enum MessageError{
    ErrorCreatingMessage,
    ErrorSendingMessage,
    ErrorCreatingHeaderMessage,
    ErrorSendingHeaderMessage,
}
/// Contains all necessary fields, for sending a version message needed for doing a handshake among nodes
pub struct VersionMessage {
    version: i32,
    services: u64,
    timestamp: i64,
    addr_recv_services: u64,
    receiver_address: [u8; 16],
    receiver_port: u16,
    addr_sender_services: u64,
    sender_address: [u8; 16],
    sender_port: u16,
    nonce: u64,
    user_agent_length: u8,
    //user_agent: String,
    start_height: i32,
    relay: u8,
}

impl Message for VersionMessage{
    fn send_to(&self, tcp_stream: &mut TcpStream)-> Result<(), MessageError> {
        let payload = self.to_bytes();
        let command_name = "version\0\0\0\0\0";
        let header_message = HeaderMessage::new(command_name, &payload)?;
        header_message.send_to(tcp_stream)?;

        match tcp_stream.write(payload.as_slice()){
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorSendingMessage),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        bytes_vector.extend_from_slice(&self.services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes_vector.extend_from_slice(&self.addr_recv_services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.receiver_address);
        bytes_vector.extend_from_slice(&self.receiver_port.to_be_bytes()); 
        bytes_vector.extend_from_slice(&self.addr_sender_services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.sender_address);
        bytes_vector.extend_from_slice(&self.sender_port.to_be_bytes());
        bytes_vector.extend_from_slice(&self.nonce.to_le_bytes());
        bytes_vector.push(self.user_agent_length);
        //bytes_vector.extend_from_slice(&self.user_agent);
        bytes_vector.extend_from_slice(&self.start_height.to_le_bytes());
        bytes_vector.push(self.relay);
        bytes_vector
    }
}

impl VersionMessage {
    /// Constructor for the struct VersionMessage, receives a version and a reciever address (which
    /// includes both the ip and port) and returns an instance of a VersionMessage with all its 
    /// necesary attributes initialized, the optional ones are left in blank
    pub fn new(version: i32, receiver_address: SocketAddr) -> Result<VersionMessage, MessageError> {
        let version_message = VersionMessage {
            version,
            services: NODE_NETWORK,
            timestamp: Utc::now().timestamp(),
            addr_recv_services: 0, //Como no sabemos que servicios admite el nodo asumimos que no admite ningun servicio
            receiver_address: {
                match receiver_address.ip(){
                    std::net::IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped().octets(),
                    std::net::IpAddr::V6(ipv6) => ipv6.octets(),
                }
            },
            receiver_port: receiver_address.port(),
            addr_sender_services: NODE_NETWORK,
            sender_address:  std::net::Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets(),
            sender_port: LOCAL_PORT,
            nonce: rand::thread_rng().gen(),
            user_agent_length: 0,
            //user_agent,   no ponemos el user agent,porque entendemos que nadie nos conoce, a nadie le va a interesar saber en que version esta papas rusticas 0.0.1
            start_height: 0,
            relay: 0x01,
        };
        Ok(version_message)
    }
}

struct HeaderMessage {
    start_string: [u8; 4],
    command_name: [u8; 12],
    payload_size: u32,
    checksum: [u8; 4],
}

impl Message for HeaderMessage{
    ///Sends a header message trough the tcp_stream
    fn send_to(&self, tcp_stream: &mut TcpStream)-> Result<(), MessageError>{
        match tcp_stream.write(self.to_bytes().as_slice()){
            Ok(_) => Ok(()),
            Err(_) => return Err(MessageError::ErrorSendingHeaderMessage),
        }
    }
    /// Returns an array of bytes with the header message in the format specified in the bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.start_string);
        bytes_vector.extend_from_slice(&self.command_name);
        bytes_vector.extend_from_slice(&self.payload_size.to_le_bytes());
        bytes_vector.extend_from_slice(&self.checksum);
        bytes_vector
    }
}
/// Constructor for the struct HeaderMessage, receives a command name and a payload size and returns
/// an instance of a HeaderMessage with all its necesary attributes initialized.
/// The checksum is calculated using the first 4 bytes of the hash of the payload size.
/// The sender address is set to the local host address.
/// The command name is set to the first 12 bytes of the command name received as a parameter,
/// if the command name is shorter than 12 bytes, the remaining bytes are filled with 0s.
/// The payload size is set to the payload size received as a parameter.
impl HeaderMessage{
    fn new(command_name: &str , payload: &Vec<u8>) -> Result<HeaderMessage,MessageError>{
        let command_bytes = command_name.as_bytes();
        let mut command_bytes_fixed_size =[0u8; 12] ;
        command_bytes_fixed_size.copy_from_slice(command_bytes);
        let payload_size = std::mem::size_of_val(payload.as_slice()) as u32;  

        let hash = sha256d::Hash::hash(payload.as_slice());
        let hash_value = hash.as_byte_array();
        let checksum: [u8; 4] = match hash_value[..4].try_into(){
            Ok(array) => array,
            Err(_) => return Err(MessageError::ErrorCreatingHeaderMessage),
        };
        
        let header_message = HeaderMessage {
            start_string: START_STRING_TEST_NET,
            command_name: command_bytes_fixed_size,
            payload_size,
            checksum, //(SHA256(SHA256(<empty string>)))
        };
        Ok(header_message)
    }
}

/// Message used to acknoledge 2 nodes have sent Version Messages.
struct VerACKMessage{

}

impl Message for VerACKMessage{
    fn send_to(&self, tcp_stream: &mut TcpStream)-> Result<(), MessageError>{
        let payload = self.to_bytes();
        let command_name = "verack\0\0\0\0\0\0";
        let header_message = HeaderMessage::new(command_name, &payload)?;
        header_message.send_to(tcp_stream)
    }
    fn to_bytes(&self) -> Vec<u8>{
        Vec::new()
    }
}

impl VerACKMessage {
    pub fn new() -> Result<VerACKMessage,MessageError> {
        Ok(VerACKMessage {  })
    }
}

trait Message{
    //pub fn new(version: i32, receiver_address: SocketAddr) -> VersionMessage;
    fn send_to(&self, tcp_stream: &mut TcpStream)-> Result<(), MessageError>;
    fn to_bytes(&self) -> Vec<u8>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version_message_expected_bytes(timestamp: i64, socket: SocketAddr, rand: u64)->Vec<u8>{
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&timestamp.to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes()); 
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&std::net::Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets());
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(0 as u8);
        //bytes_vector.extend_from_slice(&self.user_agent);
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }

    fn header_message_expected_bytes()->Vec<u8>{
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&START_STRING_TEST_NET);
        bytes_vector.extend_from_slice(&"version\0\0\0\0\0".as_bytes());
        bytes_vector.extend_from_slice(&(0 as u32).to_le_bytes());
        bytes_vector.extend_from_slice([0x5d,0xf6,0xe0,0xe2].as_slice());
        bytes_vector
    }

    #[test]
    fn test_1_to_bytes_version_message()-> Result<(), MessageError>{
        let socket = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 2)), 8080);
        let version_message = VersionMessage::new(70015, socket)?;

        let version_message_bytes = version_message.to_bytes();
        
        assert_eq!(version_message_bytes, version_message_expected_bytes(version_message.timestamp, socket, version_message.nonce));
        Ok(())
    }

    #[test]
    fn test_2_to_bytes_header_message()-> Result<(), MessageError>{
        let header_message = HeaderMessage::new("version\0\0\0\0\0", &Vec::new())?;

        let header_message_bytes = header_message.to_bytes();
        
        assert_eq!(header_message_bytes, header_message_expected_bytes());
        Ok(())
    }

    #[test]
    fn test_3_to_bytes_verack_message() -> Result<(), MessageError> {
        let verack_message = VerACKMessage::new()?;

        let verack_message_bytes = verack_message.to_bytes();

        assert_eq!(verack_message_bytes, Vec::new());
        Ok(())
    }
}