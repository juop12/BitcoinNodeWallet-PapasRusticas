use super::util::*;


const MAX_HASH_COUNT_SIZE: u64 = 0x02000000;
const GET_BLOCK_HEADERS_MSG_NAME: &str = "getheaders\0\0";


/// Message used to request a block header from a node.
#[derive(Debug, PartialEq)]
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
        HeaderMessage::new(GET_BLOCK_HEADERS_MSG_NAME, &self.to_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_hashes::{sha256d, Hash};
    use crate::mock_tcp_stream::MockTcpStream;


    // Auxiliar functions
    //=================================================================

    fn get_block_headers_message_expected_bytes() -> Vec <u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.push(1 as u8);
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector
    }

    // Tests
    //=================================================================

    #[test]
    fn test_to_bytes_6_get_block_headers_message() -> Result<(), MessageError> {
        let expected_bytes = get_block_headers_message_expected_bytes();

        let mut vec_hash = Vec::new();
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        vec_hash.push(hash);

        let get_block_headers_msg = GetBlockHeadersMessage::new(70015, vec_hash, hash);

        let get_block_headers_msg = get_block_headers_msg.to_bytes();

        assert_eq!(get_block_headers_msg, expected_bytes);
        Ok(())
    } 

    #[test]
    fn test_send_to_4_get_block_headers_message()-> Result<(), MessageError> {
        let mut stream = MockTcpStream::new();

        let mut vec_hash = Vec::new();
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        vec_hash.push(hash);

        let get_block_headers_msg = GetBlockHeadersMessage::new(70015, vec_hash,hash);
        let get_block_headers_hm = get_block_headers_msg.get_header_message()?;
        let mut expected_result = get_block_headers_hm.to_bytes();
        expected_result.extend(get_block_headers_msg.to_bytes());

        get_block_headers_msg.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }

    #[test]
    fn test_from_bytes_7_get_block_headers_message() -> Result<(), MessageError> {
        let mut vec_hash = Vec::new();
        let hash: [u8; 32] = *sha256d::Hash::hash(b"test").as_byte_array();
        vec_hash.push(hash);

        let expected_get_block_headers_msg = GetBlockHeadersMessage::new(70015, vec_hash,hash);

        let get_block_headers_msg = 
            GetBlockHeadersMessage::from_bytes(
                                    &mut expected_get_block_headers_msg
                                    .to_bytes()
                                    .as_mut_slice()
                                )?;

        assert_eq!(get_block_headers_msg, expected_get_block_headers_msg);
        Ok(())
    }
}
