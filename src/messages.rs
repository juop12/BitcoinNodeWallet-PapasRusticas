use chrono::{DateTime, Utc};
use rand::prelude::*;
use std::net::{SocketAddr, TcpStream};
use std::io::prelude::*;
use std::path::Component;
use bitcoin_hashes::{sha256d, Hash};

const NODE_NETWORK: u64 = 0x01;
const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 1001;


pub enum MessageError{
    ErrorCreatingMessage
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
    relay: bool,
}

impl Message for VersionMessage{
    fn send_to(&self, tcp_stream: TcpStream)-> Result<usize, ()>{
        let payload_size = std::mem::size_of_val(self) as u32;
        let version = "version\0\0\0\0\0";
        let header_message = HeaderMessage::new(version, payload_size);
        header_message.send_to(tcp_stream)?;
        tcp_stream.write_all(self.to_bytes().into_boxed_slice().into())
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
            relay: true,
        };
        Ok(version_message)
    }
}

struct HeaderMessage {
    sender_address: [u8; 4],
    command_name: [u8; 12],
    payload_size: u32,
    checksum: [u8; 4], 
}

impl Message for HeaderMessage{
    fn send_to(&self, tcp_stream: TcpStream)-> Result<usize, ()>{
        tcp_stream.write(self)
    }
}

impl HeaderMessage{
    fn new(command_name: &str , payload_size: u32) -> Result<HeaderMessage,MessageError>{
        let command_bytes = command_name.as_bytes();
        let mut command_bytes_fixed_size :[u8; 12];
        command_bytes_fixed_size.copy_from_slice(command_bytes);

        let hash_value = sha256d::Hash::hash(&payload_size.to_le_bytes()).as_byte_array();
        let checksum: [u8; 4] = match hash_value[..4].try_into(){
            Ok(array) => array,
            Err(_) => return Err(MessageError::ErrorCreatingMessage),
        };
        
        let header_message = HeaderMessage {
            sender_address: LOCAL_HOST,
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
        let header = HeaderMessage::new("verack\0\0\0\0\0\0", 0)?;
        let verack_message = VerACKMessage{
            header,
        };
        Ok(verack_message)
    }
}

trait Message{
    //pub fn new(version: i32, receiver_address: SocketAddr) -> VersionMessage;
    fn send_to(&self, tcp_stream: TcpStream)-> Result<usize, ()>;
    fn to_bytes(&self) -> Vec<u8>;
}