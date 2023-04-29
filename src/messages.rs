use chrono::Utc;
use rand::prelude::*;
use std::{
    io::{Read, Write},
    mem::size_of_val,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
};

use bitcoin_hashes::{sha256d, Hash};

const NODE_NETWORK: u64 = 0x01;
const START_STRING_TEST_NET: [u8; 4] = [0xd9, 0xb4, 0xeb, 0xf9];
const MESAGE_HEADER_SIZE: usize = 24;
const MINIMAL_VERSION_MESSAGE_SIZE: usize = 86;

#[derive(Debug, PartialEq)]
/// Error Struct for messages, contains customized errors for each type of message (excluding
/// VerACKMessage) and to diferenciate whether the error occured while instanciation or in
/// message sending
pub enum MessageError {
    ErrorCreatingVersionMessage,
    ErrorSendingVersionMessage,
    ErrorCreatingHeaderMessage,
    ErrorSendingHeaderMessage,
    ErrorCreatingVerAckMessage,
    ErrorSendingVerAckMessage,
}

#[derive(Debug, PartialEq)]
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
    user_agent_length: Vec<u8>,
    user_agent: Vec<u8>,
    start_height: i32,
    relay: u8,
}

/// Gets the user agent lenght based on the bytes of the slice. The size depends on waht the 
/// position 0 of the slice is, based on that it can be 1, 3, 5 or 9 bytes long.
/// Returns a tuple with the user agent length and the amount of bytes that it has
fn get_user_agent_length(slice: &[u8]) -> (Vec<u8>, usize) {
    let mut user_agent_length  = Vec::new();
    let mut amount_of_bytes= 1;
    if slice[0] == 0xfd{
        amount_of_bytes = 3;
    }
    if slice[0] == 0xfe {
        amount_of_bytes = 5;
    }
    if slice[0] == 0xff {
        amount_of_bytes = 9;
    }
    for i in 0..amount_of_bytes{
        user_agent_length.push(slice[i]);
    }
    (user_agent_length, amount_of_bytes)
}


impl Message for VersionMessage{

    type MessageType = VersionMessage;
    /// Implementation of the trait send_to for VersionMessage, recieves a TcpStream and
    /// returns a Result with either () if everything went Ok or a MessageError if either the
    /// message creation or sending failed
    /// For now, the command name is hardcoded, it's value should be set in the config file
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError> {
        let payload = self.to_bytes();
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write(payload.as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorSendingVersionMessage),
        }
    }

    /// Implementation of the trait to_bytes for VersionMessage, returns a vector of bytes
    /// with all the attributes of the struct. Both little and big endian are used, following
    /// the BTC protocol
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
        bytes_vector.extend(&self.user_agent_length);
        if self.user_agent_length[0] > 0 {
            bytes_vector.extend(&self.user_agent);
        }
        bytes_vector.extend_from_slice(&self.start_height.to_le_bytes());
        bytes_vector.push(self.relay);
        bytes_vector 
    }

    /// Functions as a wrapper of _from for VersionMessage, recieves a slice of bytes and
    /// returns a Result with either a VersionMessage if everything went Ok or a MessageError
    /// if the call to _from failed. The slice must be at least 86 bytes long (the minimum
    /// length of a VersionMessage)
    fn from_bytes(slice: &mut [u8])-> Result<Self::MessageType, MessageError>{
        if slice.len() < MINIMAL_VERSION_MESSAGE_SIZE {
            return Err(MessageError::ErrorCreatingVersionMessage);
        }
        match Self::_from_bytes(slice){
            Some(version_message) => Ok(version_message),
            None => Err(MessageError::ErrorCreatingVersionMessage),
        }
    }

    fn get_header_message(&self)->Result<HeaderMessage, MessageError>{
        HeaderMessage::new("version\0\0\0\0\0", &self.to_bytes())
    }
}

impl VersionMessage {
    /// Constructor for the struct VersionMessage, receives a version and a reciever address (which
    /// includes both the ip and port) and returns an instance of a VersionMessage with all its
    /// necesary attributes initialized, the optional ones are left in blank
    pub fn new(version: i32, receiver_address: SocketAddr, sender_address: SocketAddr) -> Result<VersionMessage, MessageError> {
        let mut user_agent_length = Vec::new();
        user_agent_length.push(0);
        let version_message = VersionMessage {
            version,
            services: NODE_NETWORK,
            timestamp: Utc::now().timestamp(),
            addr_recv_services: 0, //Como no sabemos que servicios admite el nodo asumimos que no admite ningun servicio
            receiver_address: {
                match receiver_address.ip() {
                    IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped().octets(),
                    IpAddr::V6(ipv6) => ipv6.octets(),
                }
            },
            receiver_port: receiver_address.port(),
            addr_sender_services: NODE_NETWORK,
            //sender_address: Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets(),
            sender_address: {
                match sender_address.ip() {
                    IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped().octets(),
                    IpAddr::V6(ipv6) => ipv6.octets(),
                }
            },
            sender_port: sender_address.port(),
            nonce: rand::thread_rng().gen(),
            user_agent_length,
            user_agent: Vec::new(),   //no ponemos el user agent,porque entendemos que nadie nos conoce, a nadie le va a interesar saber en que version esta papas rusticas 0.0.1
            start_height: 0,
            relay: 0x01,
        };
        Ok(version_message)
    }

    /// Implementation of the trait _from for VersionMessage. Recieves a slice of bytes and
    /// returns an Option with either a VersionMessage if everything went Ok or None if any step
    /// in the middle of the conversion from bytes to VersionMessage fields failed.
    fn _from_bytes(slice: &mut [u8]) -> Option<VersionMessage>{
        
        let (user_agent_length, cant_bytes) = get_user_agent_length(&slice[80..]);

        let version_message = VersionMessage {
            version: i32::from_le_bytes(slice[0..4].try_into().ok()?),
            services: u64::from_le_bytes(slice[4..12].try_into().ok()?),
            timestamp: i64::from_le_bytes(slice[12..20].try_into().ok()?),
            addr_recv_services: u64::from_le_bytes(slice[20..28].try_into().ok()?),
            receiver_address: slice[28..44].try_into().ok()?,
            receiver_port: u16::from_be_bytes(slice[44..46].try_into().ok()?),
            addr_sender_services: u64::from_le_bytes(slice[46..54].try_into().ok()?),
            sender_address: slice[54..70].try_into().ok()?,
            sender_port: u16::from_be_bytes(slice[70..72].try_into().ok()?),
            nonce: u64::from_le_bytes(slice[72..80].try_into().ok()?),
            user_agent_length,
            user_agent: Vec::from(&slice[(80 + cant_bytes)..(slice.len()-5)]),
            start_height: i32::from_le_bytes(slice[(slice.len()-5)..(slice.len()-1)].try_into().ok()?),
            relay: slice[slice.len() - 1],
        };
        Some(version_message)
    }
}

/// Contains all necessary fields for the HeaderMessage to work properly
#[derive(Debug, PartialEq, Clone)]
pub struct HeaderMessage {
    start_string: [u8; 4],
    command_name: [u8; 12],
    payload_size: u32,
    checksum: [u8; 4],
}

impl Message for HeaderMessage {
    
    type MessageType = HeaderMessage;
    /// Sends a header message trough the tcp_stream
    fn send_to<T: Read + Write>(&self, reciever_stream: &mut T) -> Result<(), MessageError> {
        match reciever_stream.write(self.to_bytes().as_slice()) {
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

    /// Receives a slice of bytes and returns a HeaderMessage if everything went Ok or a
    /// MessageError if the conversion from bytes to HeaderMessage failed.
    fn from_bytes(slice: &mut [u8])-> Result<Self::MessageType, MessageError>{
        if slice.len() != MESAGE_HEADER_SIZE {
            return Err(MessageError::ErrorCreatingHeaderMessage);
        }
        match Self::_from_bytes(slice){
            Some(header_message) => Ok(header_message),
            None => Err(MessageError::ErrorCreatingHeaderMessage),
        }
    }

    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        Ok(self.clone())
    }
}
/// Constructor for the struct HeaderMessage, receives a command name and a payload size and returns
/// an instance of a HeaderMessage with all its necesary attributes initialized, according to the
/// p2p bitcoing protocol
impl HeaderMessage {
    fn new(command_name: &str, payload: &Vec<u8>) -> Result<HeaderMessage, MessageError> {
        let command_bytes = command_name.as_bytes();
        let mut command_bytes_fixed_size = [0u8; 12];
        command_bytes_fixed_size.copy_from_slice(command_bytes);
        let payload_size = size_of_val(payload.as_slice()) as u32;

        let hash = sha256d::Hash::hash(payload.as_slice());
        let hash_value = hash.as_byte_array();
        let checksum: [u8; 4] = match hash_value[..4].try_into() {
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

    pub fn get_payload_size(&self) ->  u32 {
        self.payload_size
    }

    pub fn get_command_name(&self) -> [u8; 12] {
        self.command_name
    }

    /// Receives a slice of bytes and returns an Option<HeaderMessage>, initialices the fields of
    /// the HeaderMessage with the values in the slice, if any step in the middle of the conversion
    /// fails, returns None.
    fn _from_bytes(slice: &mut [u8]) -> Option<HeaderMessage>{   
        
        let start_string = slice[0..4].try_into().ok()?;
        let command_name = slice[4..16].try_into().ok()?;
        let payload_size = u32::from_le_bytes(slice[16..20].try_into().ok()?);
        let checksum = slice[20..24].try_into().ok()?;
        Some(HeaderMessage {
            start_string,
            command_name,
            payload_size,
            checksum,
        })
    }
}

/// Message used to acknoledge 2 nodes have sent Version Messages.
#[derive(Debug, PartialEq)]
pub struct VerACKMessage {}

impl Message for VerACKMessage {

    type MessageType = VerACKMessage;
    /// Implements the trait send_to for VerACKMessage, sends a VerACKMessage trough the tcp_stream,
    /// returns an error if the message could not be sent.
    fn send_to<T: Read + Write>(&self, reciever_stream: &mut T) -> Result<(), MessageError> {
        let header_message = self.get_header_message()?;
        header_message.send_to(reciever_stream)
    }
    /// Returns an empty vector of bytes, since the VerACKMessage has no payload.
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Returns a VerACKMessage if the slice of bytes is empty, otherwise returns a MessageError.
    fn from_bytes(slice: &mut [u8])-> Result<Self::MessageType, MessageError>{
        if slice.len() != 0 {
            return Err(MessageError::ErrorCreatingVerAckMessage);
        }
        Ok(VerACKMessage{})
    }

    fn get_header_message(&self)->Result<HeaderMessage, MessageError>{
        HeaderMessage::new("verack\0\0\0\0\0\0", &self.to_bytes())
    }
}

impl VerACKMessage {
    /// Constructor for the struct VerACKMessage, returns an instance of a VerACKMessage
    pub fn new() -> Result<VerACKMessage, MessageError> {
        Ok(VerACKMessage {})
    }
}

//Hacer un wrapper para send to,cosa de que solo se pueda mandar un tcpStream?
pub trait Message {

    type MessageType;
    //Writes the message as bytes in the reciever_stream
    fn send_to<T: Read + Write>(&self, reciever_stream: &mut T) -> Result<(), MessageError>;

    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>;
    
    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8])-> Result<Self::MessageType, MessageError>;

    fn get_header_message(&self)->Result<HeaderMessage, MessageError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self};

    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001;

    fn version_message_without_user_agent_expected_bytes(timestamp: i64, rand: u64) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&timestamp.to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes());
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(
            &Ipv4Addr::from(LOCAL_HOST)
                .to_ipv6_mapped()
                .octets(),
        );
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(0 as u8);
        //bytes_vector.extend_from_slice(&self.user_agent);
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }
    
    fn version_message_with_user_agent_expected_bytes() -> Vec<u8> {
        let rand: u64= rand::thread_rng().gen();
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&(Utc::now().timestamp() as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes());
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(
            &Ipv4Addr::from(LOCAL_HOST)
                .to_ipv6_mapped()
                .octets(),
        );
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(253 as u8);
        bytes_vector.extend_from_slice(&(4 as u16).to_le_bytes());
        bytes_vector.extend_from_slice(&"test".as_bytes());
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }

    fn empty_header_message_expected_bytes() -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&START_STRING_TEST_NET);
        bytes_vector.extend_from_slice(&"verack\0\0\0\0\0\0".as_bytes());
        bytes_vector.extend_from_slice(&(0 as u32).to_le_bytes());
        bytes_vector.extend_from_slice([0x5d, 0xf6, 0xe0, 0xe2].as_slice());
        bytes_vector
    }

    fn non_empty_header_message_expected_bytes() -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&START_STRING_TEST_NET);
        bytes_vector.extend_from_slice(&"n_empty\0\0\0\0\0".as_bytes());
        bytes_vector.extend_from_slice(&(4 as u32).to_le_bytes());
        bytes_vector.extend_from_slice([0x8d, 0xe4, 0x72, 0xe2].as_slice());
        bytes_vector
    }

    #[test]
    fn test_to_bytes_1_version_message_without_user_agent() -> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127,0,0,2], 8080));
        let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
        let version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;

        let version_message_bytes = version_message.to_bytes();

        assert_eq!(
            version_message_bytes,
            version_message_without_user_agent_expected_bytes(
                version_message.timestamp,
                version_message.nonce
            )
        );
        Ok(())
    }

    #[test]
    fn test_to_bytes_2_empty_header_message() -> Result<(), MessageError> {
        let header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;

        let header_message_bytes = header_message.to_bytes();

        assert_eq!(header_message_bytes, empty_header_message_expected_bytes());
        Ok(())
    }

    #[test]

    fn test_to_bytes_3_non_empty_header_message() -> Result<(), MessageError> {
        let header_message = HeaderMessage::new("n_empty\0\0\0\0\0", &vec![1, 2, 3, 4])?;

        let header_message_bytes = header_message.to_bytes();

        assert_eq!(
            header_message_bytes,
            non_empty_header_message_expected_bytes()
        );
        Ok(())
    }
    #[test]
    fn test_to_bytes_4_verack_message() -> Result<(), MessageError> {
        let verack_message = VerACKMessage::new()?;

        let verack_message_bytes = verack_message.to_bytes();

        assert_eq!(verack_message_bytes, Vec::new());
        Ok(())
    }
    #[test]
    fn test_to_bytes_5_version_message_with_user_agent()-> Result<(), MessageError> {

        let mut expected_bytes = version_message_with_user_agent_expected_bytes();
        let version_message = VersionMessage::from_bytes(&mut expected_bytes.as_mut_slice())?;

        let version_message_bytes = version_message.to_bytes();

        assert_eq!(version_message_bytes, expected_bytes);
        Ok(())
    }

    /// Has both read and write buffers to test if the messages are correctly sent
    struct MockTcpStream {
        read_buffer: Vec<u8>,
        write_buffer: Vec<u8>,
    }

    impl MockTcpStream {
        /// Constructor for MockTcpStream
        fn new() -> MockTcpStream {
            MockTcpStream {
                read_buffer: Vec::new(),
                write_buffer: Vec::new(),
            }
        }
    }

    impl Read for MockTcpStream {
        /// Reads bytes from the stream until completing the buffer and returns how many bytes were read
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.read_buffer.as_slice().read(buf)
        }
    }

    impl Write for MockTcpStream {
        /// Writes the buffer value on the stream and returns how many bytes were written
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_buffer.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.write_buffer.flush()
        }
    }

    #[test]
    fn test_send_to_1_header_message() -> Result<(), MessageError> {
        let header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;
        let mut stream = MockTcpStream::new();

        header_message.send_to(&mut stream);

        assert_eq!(stream.write_buffer, header_message.to_bytes());
        Ok(())
    }

    #[test]
    fn test_send_to_2_version_message() -> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127,0,0,2], 8080));
        let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
        let version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;
        let header_message = version_message.get_header_message()?;
        let mut stream = MockTcpStream::new();
        let mut expected_result = header_message.to_bytes();
        expected_result.extend(version_message.to_bytes());

        version_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }

    #[test]
    fn test_send_to_3_version_message() -> Result<(), MessageError> {
        let ver_ack_message = VerACKMessage::new()?;
        let mut stream = MockTcpStream::new();

        ver_ack_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, empty_header_message_expected_bytes());
        Ok(())
    }

    #[test]
    fn test_from_bytes_1_without_user_agent_version_message()-> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127,0,0,2], 8080));
        let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
        let expected_version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;

        let version_message = VersionMessage::from_bytes(&mut expected_version_message.to_bytes().as_mut_slice())?;

        assert_eq!(version_message, expected_version_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_2_with_user_agent_version_message()-> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127,0,0,2], 8080));
        let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
        let mut expected_version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;
        expected_version_message.user_agent_length = vec![253, 4, 0];
        expected_version_message.user_agent = Vec::from("test");

        let version_message = VersionMessage::from_bytes(&mut expected_version_message.to_bytes().as_mut_slice())?;

        assert_eq!(version_message, expected_version_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_3_empty_header_message() -> Result<(), MessageError> {
        let expected_header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;

        let header_message = HeaderMessage::from_bytes(&mut expected_header_message.to_bytes().as_mut_slice())?;

        assert_eq!(header_message, header_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_4_non_empty_header_message() -> Result<(), MessageError> {
        let expected_header_message = HeaderMessage::new("version\0\0\0\0\0", &vec![1, 2, 3, 4])?;

        let header_message = HeaderMessage::from_bytes(&mut expected_header_message.to_bytes().as_mut_slice())?;

        assert_eq!(header_message, expected_header_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_5_verack_message_from_empty_slice() -> Result<(), MessageError> {
        let expected_verack_message = VerACKMessage::new()?;

        let verack_message = VerACKMessage::from_bytes(&mut expected_verack_message.to_bytes().as_mut_slice())?;

        assert_eq!(verack_message, expected_verack_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_6_verack_message_from_non_empty_slice() -> Result<(), MessageError> {
        let expected_verack_message = VerACKMessage::new()?;
        let mut expected_bytes = expected_verack_message.to_bytes();
        expected_bytes.extend(vec![1, 2, 3, 4]);

        let verack_message = VerACKMessage::from_bytes(&mut expected_bytes.as_mut_slice()).unwrap_err();
        
        assert_eq!(verack_message, MessageError::ErrorCreatingVerAckMessage);
        Ok(())
    }

}
