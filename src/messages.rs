use chrono::Utc;
use rand::prelude::*;
use crate::blockchain::*;
use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr},
}; 


use bitcoin_hashes::{sha256d, Hash};

const NODE_NETWORK: u64 = 0x01;
const START_STRING_TEST_NET: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];
const MESAGE_HEADER_SIZE: usize = 24;
const MINIMAL_VERSION_MESSAGE_SIZE: usize = 86;
const COMMAND_NAME_ERROR: &str = "\0\0\0\0\0\0\0\0\0\0\0\0";

//====================================================================================
//====================================================================================

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
    ErrorCreatingGetBlockHeadersMessage,
    ErrorSendingGetBlockHeadersMessage,
    ErrorCreatingBlockHeadersMessage,
    ErrorHeadersBlockMessage,
}

//====================================================================================
//====================================================================================

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

//Hacer refactor para que verifique que recibio la cantidad de datos que corresponde. Ya sea chequeandolo aca o devolviendo la cantidad

/// Gets the variable lenght based on the bytes of the slice. The size depends on what the
/// position 0 of the slice is, based on that it can be 1, 3, 5 or 9 bytes long.
/// Returns a tuple with the variable length and the amount of bytes that it has
fn calculate_variable_length_integer(slice: &[u8]) -> (Vec<u8>, usize) {
    let mut length = Vec::new();
    let mut amount_of_bytes = 1;
    if slice[0] == 0xfd {
        amount_of_bytes = 3;
    }
    if slice[0] == 0xfe {
        amount_of_bytes = 5;
    }
    if slice[0] == 0xff {
        amount_of_bytes = 9;
    }
    for i in slice.iter().take(amount_of_bytes) {
        length.push(*i);
    }
    (length, amount_of_bytes)
}

impl Message for VersionMessage {
    type MessageType = VersionMessage;
    /// Implementation of the trait send_to for VersionMessage, recieves a TcpStream and
    /// returns a Result with either () if everything went Ok or a MessageError if either the
    /// message creation or sending failed
    /// For now, the command name is hardcoded, it's value should be set in the config file
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError> {
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write(self.to_bytes().as_slice()) {
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
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError> {
        if slice.len() < MINIMAL_VERSION_MESSAGE_SIZE {
            return Err(MessageError::ErrorCreatingVersionMessage);
        }

        match Self::_from_bytes(slice) {
            Some(version_message) => Ok(version_message),
            None => Err(MessageError::ErrorCreatingVersionMessage),
        }
    }

    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("version\0\0\0\0\0", &self.to_bytes())
    }
}

impl VersionMessage {
    /// Constructor for the struct VersionMessage, receives a version and a receiver address (which
    /// includes both the ip and port) and returns an instance of a VersionMessage with all its
    /// necesary attributes initialized, the optional ones are left in blank
    pub fn new(
        version: i32,
        receiver_address: SocketAddr,
        sender_address: SocketAddr,
    ) -> Result<VersionMessage, MessageError> {
        let user_agent_length = vec![0];
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
            sender_address: {
                match sender_address.ip() {
                    IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped().octets(),
                    IpAddr::V6(ipv6) => ipv6.octets(),
                }
            },
            sender_port: sender_address.port(),
            nonce: rand::thread_rng().gen(),
            user_agent_length,
            user_agent: Vec::new(),
            start_height: 0,
            relay: 0x01,
        };

        Ok(version_message)
    }

    /// Implementation of the trait _from for VersionMessage. Recieves a slice of bytes and
    /// returns an Option with either a VersionMessage if everything went Ok or None if any step
    /// in the middle of the conversion from bytes to VersionMessage fields failed.
    fn _from_bytes(slice: &mut [u8]) -> Option<VersionMessage> {
        if slice[80..].len() <= 0{
            return None;
        }
        let (user_agent_length, cant_bytes) = calculate_variable_length_integer(&slice[80..]);

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
            user_agent: Vec::from(&slice[(80 + cant_bytes)..(slice.len() - 5)]),
            start_height: i32::from_le_bytes(
                slice[(slice.len() - 5)..(slice.len() - 1)]
                    .try_into()
                    .ok()?,
            ),
            relay: slice[slice.len() - 1],
        };
        Some(version_message)
    }
}

//====================================================================================
//====================================================================================


/// Struct that represents a header message in the bitcoin protocol
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
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError> {
        match receiver_stream.write(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorSendingHeaderMessage),
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
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError> {
        if slice.len() != MESAGE_HEADER_SIZE {
            return Err(MessageError::ErrorCreatingHeaderMessage);
        }
        match Self::_from_bytes(slice) {
            Some(header_message) => Ok(header_message),
            None => Err(MessageError::ErrorCreatingHeaderMessage),
        }
    }

    //Returns a copy of the header message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        Ok(self.clone())
    }
}

impl HeaderMessage {
    /// Receives a command name and a payload size and returns an instance of a HeaderMessage with
    /// all its necesary attributes initialized, according to the p2p bitcoin protocol
    pub fn new(command_name: &str, payload: &Vec<u8>) -> Result<HeaderMessage, MessageError> {
        if command_name.len() > 12 {
            return Err(MessageError::ErrorCreatingHeaderMessage);
        }

        let mut command_bytes = Vec::from(command_name.as_bytes());
        while command_bytes.len() < 12 {
            command_bytes.push(0);
        }
        
        let mut command_bytes_fixed_size = [0u8; 12];
        command_bytes_fixed_size.copy_from_slice(command_bytes.as_slice());
        //let payload_size = size_of_val(payload.as_slice()) as u32;

        let payload_size: u32 = payload.len() as u32;

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

    /// Returns the payload size of the header message
    pub fn get_payload_size(&self) -> u32 {
        self.payload_size
    }

    /// Returns the command name of the header message
    pub fn get_command_name(&self) -> String {
        match String::from_utf8(Vec::from(self.command_name)) {
            Ok(string) => string,
            Err(_) => String::from(COMMAND_NAME_ERROR),
        }
    }

    /// Receives a slice of bytes and returns an Option<HeaderMessage>, initialices the fields of
    /// the HeaderMessage with the values in the slice, if any step in the middle of the conversion
    /// fails, returns None.
    fn _from_bytes(slice: &mut [u8]) -> Option<HeaderMessage> {
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

//====================================================================================
//====================================================================================

/// Message used to acknoledge 2 nodes have sent Version Messages.
#[derive(Debug, PartialEq)]
pub struct VerACKMessage {}

impl Message for VerACKMessage {
    type MessageType = VerACKMessage;
    /// Implements the trait send_to for VerACKMessage, sends a VerACKMessage trough the tcp_stream,
    /// returns an error if the message could not be sent.
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError> {
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)
    }
    /// Returns an empty vector of bytes, since the VerACKMessage has no payload.
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Returns a VerACKMessage if the slice of bytes is empty, otherwise returns a MessageError.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError> {
        if !slice.is_empty() {
            return Err(MessageError::ErrorCreatingVerAckMessage);
        }
        Ok(VerACKMessage {})
    }

    /// Returns a copy of the header message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("verack\0\0\0\0\0\0", &self.to_bytes())
    }
}

impl VerACKMessage {
    /// Constructor for the struct VerACKMessage, returns an instance of a VerACKMessage
    pub fn new() -> Result<VerACKMessage, MessageError> {
        Ok(VerACKMessage {})
    }
}

//====================================================================================
//====================================================================================

const MAX_HASH_COUNT_SIZE: u64 = 0x02000000;
#[derive(Debug, PartialEq)]
/// Message used to request a block header from a node.
pub struct GetBlockHeadersMessage {
    version: u32,
    hash_count: Vec<u8>, //Que pasa si es 0?
    block_header_hashes: Vec<[u8; 32]>, //Que pasa si su cantidad es distinta de hash_count?
    stopping_hash: [u8; 32],
}

impl Message for GetBlockHeadersMessage{

    type MessageType = GetBlockHeadersMessage;
    
    /// Sends a HeaderMessage and a GetBlockHeadersMessage through the tcp_stream. On success,
    /// returns (), otherwise returns an error.
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>{
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorSendingGetBlockHeadersMessage),
       }
    }

    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        println!("\n\n\n{:?}\n\n\n\n", self.hash_count);

        bytes_vector.extend(&self.hash_count);

        for i in 0..self.block_header_hashes.len(){
            bytes_vector.extend(&self.block_header_hashes[i as usize]);
        }

        bytes_vector.extend_from_slice(&self.stopping_hash);
        bytes_vector
    }

    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError>{
        if slice.len() <= 0{
            return Err(MessageError::ErrorCreatingGetBlockHeadersMessage);
        }
        
        //checkear el tamaño max de la tira de bytes.
        match Self::_from_bytes(slice) {
            Some(get_header_message) => Ok(get_header_message),
            None => Err(MessageError::ErrorCreatingGetBlockHeadersMessage),
        }
    }
    
    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        HeaderMessage::new("getheaders\0\0", &self.to_bytes())
    }
} 

impl GetBlockHeadersMessage{

    /// Rreturns an instance of a GetBlockHeadersMessage
    pub fn new(version: u32, block_header_hashes: Vec<[u8;32]>, stopping_hash: [u8; 32]) -> GetBlockHeadersMessage{
        let mut hash_count = Vec::new();
        hash_count.push(block_header_hashes.len() as u8); //suponemos que nuca vamos a querer mas de 253 sin incluir
        GetBlockHeadersMessage{
            version,
            hash_count, 
            block_header_hashes,
            stopping_hash,
        }
    }

    /// Receives a slice of bytes and returns a GetBlockHeadersMessage. If anything fails, None
    /// is returned.
    fn _from_bytes(slice: &mut [u8]) -> Option<GetBlockHeadersMessage> {
        
        let (hash_count, cant_bytes) = calculate_variable_length_integer(&slice[4..]);
        let stopping_hash_length = slice.len() - 32;
        
        let version = u32::from_le_bytes(slice[0..4].try_into().ok()?);
        let block_header_hashes_bytes = Vec::from(&slice[(4 + cant_bytes)..stopping_hash_length]);
        
        let mut aux = 4 + cant_bytes;
        let mut block_header_hashes :Vec<[u8;32]> = Vec::new();
        while aux < stopping_hash_length{
            let a :[u8;32] = slice[aux..(aux+32)].try_into().ok()?;
            block_header_hashes.push(a);
            aux += 32;
        }
        
        //p no hay que chequear que sea 0?
        let stopping_hash = slice[stopping_hash_length..].try_into().ok()?;

        Some(GetBlockHeadersMessage{
            version,
            hash_count,
            block_header_hashes,
            stopping_hash,
        })
    }
}

//====================================================================================
//====================================================================================

const BLOCKHEADERSIZE: usize = 80;

/// The BlockHeader struct represents a block header in the Bitcoin network.
#[derive(Debug, PartialEq)]
pub struct BlockHeadersMessage {
    count: Vec<u8>,
    headers: Vec<BlockHeader>,
}

impl Message for BlockHeadersMessage{

    type MessageType = BlockHeadersMessage;
    
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>{
        todo!()
    }
    
    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        let mut bytes_vector = self.count.clone();
        for header in &self.headers{
            bytes_vector.extend(header.to_bytes());
        }
        bytes_vector
    }

    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError>{
            if slice.len() <= 0{
                return Err(MessageError::ErrorCreatingBlockHeadersMessage);
            }
        
            match Self::_from_bytes(slice) {
                Some(get_header_message) => Ok(get_header_message),
                None => Err(MessageError::ErrorCreatingBlockHeadersMessage),
            }
    }
    
    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        HeaderMessage::new("headers\0\0", &self.to_bytes())
    }
} 

impl BlockHeadersMessage{

    pub fn new(headers: Vec<BlockHeader>) -> BlockHeadersMessage{
        let mut count = Vec::new();
        count.push(headers.len() as u8); //estamos asumiendo que solo van de 253 a menor
        BlockHeadersMessage{
            count,
            headers,
        }
    }
    /*
    pub struct BlockHeadersMessage {
        count: Vec<u8>,
        headers: Vec<BlockHeader>,
    }*/

    fn _from_bytes(slice: &mut [u8]) -> Option<BlockHeadersMessage> {
        let (count, amount_of_bytes) = calculate_variable_length_integer(&slice);
        
        /* 
        if (amount_of_headers * 80 + count.len()) != slice.len(){
            let a =  amount_of_headers * 80 + count.len();
            let b = slice.len();
            return None;
        }*/
        
        let mut headers :Vec<BlockHeader> = Vec::new();
        let first_header_position = count.len();

        let mut i = count.len();
        while i < slice.len(){
            let mut block_headers_bytes = Vec::from(&slice[(i)..(i + 80)]);
            let bloc_header = BlockHeader::from_bytes(&mut block_headers_bytes).ok()?;
            headers.push(bloc_header);
            i += 80;
        }
        Some(BlockHeadersMessage::new(headers))
    }


    pub fn collect_in_vector(&self, blocks_headers: &Vec<BlockHeader>) -> Result<(), MessageError> {
        todo!();
        /*
        self.headers.iter().for_each( |block_header| {
            let bh = match BlockHeader::from_bytes(block_header.as_mut_slice()) {
                Ok(bh) => bh,
                Err(_) => return (),
            };

            blocks_headers.push(bh);
        });
        
        Ok(());
        */
    }
    /* 
    pub fn get_count(&self) -> Vec<u8> {
        self.count
    }*/
}

//====================================================================================
//====================================================================================

//Hacer un wrapper para send to,cosa de que solo se pueda mandar un tcpStream?
pub trait Message {
    type MessageType;
    //Writes the message as bytes in the receiver_stream
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>;

    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>;

    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError>;

    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>;
}

//====================================================================================
//====================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_tcp_stream::*;
    use std::net::Ipv4Addr;

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
        bytes_vector.extend_from_slice(&Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets());
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(0 as u8);
        //bytes_vector.extend_from_slice(&self.user_agent);
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }

    fn version_message_with_user_agent_expected_bytes() -> Vec<u8> {
        let rand: u64 = rand::thread_rng().gen();
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&(Utc::now().timestamp() as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes());
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets());
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

    fn get_block_headers_message_expected_bytes() -> Vec <u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.push(1 as u8);
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector
    }

    fn block_headers_message_expected_bytes() -> (Vec<u8>, BlockHeader, BlockHeader){
        let hash1 :[u8;32] = *sha256d::Hash::hash(b"test1").as_byte_array();
        let hash2 :[u8;32] = *sha256d::Hash::hash(b"test2").as_byte_array();
        let merkle_hash1 :[u8;32] = *sha256d::Hash::hash(b"test merkle root1").as_byte_array();
        let merkle_hash2 :[u8;32] = *sha256d::Hash::hash(b"test merkle root2").as_byte_array();
        
        let b_h1 = BlockHeader::new(70015, hash1, merkle_hash1); 
        let b_h2 = BlockHeader::new(70015, hash2, merkle_hash2);

        let mut expected_bytes = Vec::new();
        expected_bytes.push(2);
        expected_bytes.extend(b_h1.to_bytes());
        expected_bytes.extend(b_h2.to_bytes());
        (expected_bytes, b_h1, b_h2)
    }
   
    #[test]
    fn test_to_bytes_1_version_message_without_user_agent() -> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
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
    fn test_to_bytes_5_version_message_with_user_agent() -> Result<(), MessageError> {
        let mut expected_bytes = version_message_with_user_agent_expected_bytes();
        let version_message = VersionMessage::from_bytes(&mut expected_bytes.as_mut_slice())?;

        let version_message_bytes = version_message.to_bytes();

        assert_eq!(version_message_bytes, expected_bytes);
        Ok(())
    }

    #[test]
    fn test_to_bytes_6_get_block_headers_message() -> Result<(), MessageError> {
        let expected_bytes = get_block_headers_message_expected_bytes();
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let mut vec_hash = Vec::new();
        vec_hash.push(hash);
        let get_block_headers_message = GetBlockHeadersMessage::new(70015, vec_hash,hash);

        let get_block_headers_message = get_block_headers_message.to_bytes();

        assert_eq!(get_block_headers_message, expected_bytes);
        Ok(())
    } 

    #[test]
    fn test_to_bytes_8_block_headers_message() -> Result<(), MessageError> {
        
        let (expected_bytes, b_h1, b_h2) = block_headers_message_expected_bytes();
        let mut block_headers = Vec::new();
        block_headers.push(b_h1);
        block_headers.push(b_h2);

        let block_headers_message = BlockHeadersMessage::new(block_headers);

        assert_eq!(block_headers_message.to_bytes(), expected_bytes);
        Ok(())
    }

    #[test]
    fn test_send_to_1_header_message() -> Result<(), MessageError> {
        let header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;
        let mut stream = MockTcpStream::new();

        header_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, header_message.to_bytes());
        Ok(())
    }

    #[test]
    fn test_send_to_2_version_message() -> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
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
    fn test_send_to_4_get_block_headers_message()-> Result<(), MessageError> {
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let mut vec_hash = Vec::new();
        vec_hash.push(hash);
        let get_block_headers_message = GetBlockHeadersMessage::new(70015, vec_hash,hash);
        let mut stream = MockTcpStream::new();
        let header_message = get_block_headers_message.get_header_message()?;
        let mut expected_result = header_message.to_bytes();
        expected_result.extend(get_block_headers_message.to_bytes());
        get_block_headers_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }

    #[test]
    fn test_from_bytes_1_without_user_agent_version_message() -> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
        let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
        let expected_version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;

        let version_message =
            VersionMessage::from_bytes(&mut expected_version_message.to_bytes().as_mut_slice())?;

        assert_eq!(version_message, expected_version_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_2_with_user_agent_version_message() -> Result<(), MessageError> {
        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
        let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
        let mut expected_version_message =
            VersionMessage::new(70015, receiver_socket, sender_socket)?;
        expected_version_message.user_agent_length = vec![253, 4, 0];
        expected_version_message.user_agent = Vec::from("test");

        let version_message =
            VersionMessage::from_bytes(&mut expected_version_message.to_bytes().as_mut_slice())?;

        assert_eq!(version_message, expected_version_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_3_empty_header_message() -> Result<(), MessageError> {
        let expected_header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;

        let header_message =
            HeaderMessage::from_bytes(&mut expected_header_message.to_bytes().as_mut_slice())?;

        assert_eq!(header_message, header_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_4_non_empty_header_message() -> Result<(), MessageError> {
        let expected_header_message = HeaderMessage::new("version\0\0\0\0\0", &vec![1, 2, 3, 4])?;

        let header_message =
            HeaderMessage::from_bytes(&mut expected_header_message.to_bytes().as_mut_slice())?;

        assert_eq!(header_message, expected_header_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_5_verack_message_from_empty_slice() -> Result<(), MessageError> {
        let expected_verack_message = VerACKMessage::new()?;

        let verack_message =
            VerACKMessage::from_bytes(&mut expected_verack_message.to_bytes().as_mut_slice())?;

        assert_eq!(verack_message, expected_verack_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_6_verack_message_from_non_empty_slice() -> Result<(), MessageError> {
        let expected_verack_message = VerACKMessage::new()?;
        let mut expected_bytes = expected_verack_message.to_bytes();
        expected_bytes.extend(vec![1, 2, 3, 4]);

        let verack_message =
            VerACKMessage::from_bytes(&mut expected_bytes.as_mut_slice()).unwrap_err();

        assert_eq!(verack_message, MessageError::ErrorCreatingVerAckMessage);
        Ok(())
    }

    #[test]
    fn test_from_bytes_7_get_block_headers_message() -> Result<(), MessageError> {
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let mut vec_hash = Vec::new();
        vec_hash.push(hash);
        let expected_get_block_headers_message = GetBlockHeadersMessage::new(70015, vec_hash,hash);

        let  get_block_headers_message=
        GetBlockHeadersMessage::from_bytes(&mut expected_get_block_headers_message.to_bytes().as_mut_slice())?;

        assert_eq!(get_block_headers_message, expected_get_block_headers_message);
        Ok(())
    }

    #[test]
    fn test_from_bytes_8_block_headers_message() -> Result<(), MessageError> {
        let (mut expected_bytes, b_h1, b_h2) = block_headers_message_expected_bytes();
        let mut block_headers = Vec::new();
        block_headers.push(b_h1);
        block_headers.push(b_h2);

        let expected_block_headers_message = BlockHeadersMessage::new(block_headers);

        let block_headers_message = BlockHeadersMessage::from_bytes(&mut expected_bytes)?;
        assert_eq!(block_headers_message, expected_block_headers_message);
        Ok(())

    }
}
