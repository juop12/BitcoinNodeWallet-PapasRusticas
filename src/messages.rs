use chrono::{Utc};
use rand::prelude::*;
use std::net::{SocketAddr, TcpStream};
use std::io::prelude::*;
use bitcoin_hashes::{sha256d, Hash};

const NODE_NETWORK: u64 = 0x01;
const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 1001;
const START_STRING_TEST_NET : [u8; 4] = [0xd9,0xb4,0xeb,0xf9];


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
        let payload_size = std::mem::size_of_val(self) as u32;
        let command_name = "version\0\0\0\0\0";
        let header_message = HeaderMessage::new(command_name, payload_size)?;
        header_message.send_to(tcp_stream)?;

        match tcp_stream.write(self.to_bytes().as_slice()){
            Ok(_) => Ok(()),
            Err(_) => return Err(MessageError::ErrorSendingMessage),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        bytes_vector.extend_from_slice(&self.services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes_vector.extend_from_slice(&self.addr_recv_services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.receiver_address);
        bytes_vector.extend_from_slice(&self.receiver_port.to_le_bytes());
        bytes_vector.extend_from_slice(&self.addr_sender_services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.sender_address);
        bytes_vector.extend_from_slice(&self.sender_port.to_le_bytes());
        bytes_vector.extend_from_slice(&self.nonce.to_le_bytes());
        bytes_vector.extend_from_slice(&self.user_agent_length.to_le_bytes());
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
                    std::net::IpAddr::V6(ipv6) => ipv6.octets(), //deberiamos tirar error si es ipv6?
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
    fn new(command_name: &str , payload_size: u32) -> Result<HeaderMessage,MessageError>{
        let command_bytes = command_name.as_bytes();
        let mut command_bytes_fixed_size =[0u8; 12] ;
        command_bytes_fixed_size.copy_from_slice(command_bytes);

        let hash = sha256d::Hash::hash(&payload_size.to_le_bytes());
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
    header: HeaderMessage,
}

impl VerACKMessage {
    pub fn new() -> Result<VerACKMessage,MessageError> {
        match HeaderMessage::new("verack\0\0\0\0\0\0", 0){
            Ok(header) => Ok(VerACKMessage{header}),
            Err(_) => Err(MessageError::ErrorCreatingMessage),
        }
    }
}

trait Message{
    //pub fn new(version: i32, receiver_address: SocketAddr) -> VersionMessage;
    fn send_to(&self, tcp_stream: &mut TcpStream)-> Result<(), MessageError>;
    fn to_bytes(&self) -> Vec<u8>;
}