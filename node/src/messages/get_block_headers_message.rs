use super::message_trait::*;
use crate::utils::variable_length_integer::VarLenInt;

const MINIMUM_ANOMUNT_OF_BYTES: usize = 37;
pub const MAX_QUANTITY_FOR_GET_HEADERS: usize = 2000;

/// Message used to request a block header from a node.
#[derive(Debug, PartialEq)]
pub struct GetBlockHeadersMessage {
    version: u32,
    hash_count: VarLenInt,              
    pub block_header_hashes: Vec<[u8; 32]>, 
    pub stopping_hash: [u8; 32],
}

impl Message for GetBlockHeadersMessage {
    type MessageType = GetBlockHeadersMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingGetBlockHeadersMessage;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());

        bytes_vector.extend(&self.hash_count.to_bytes());

        for i in 0..self.block_header_hashes.len() {
            bytes_vector.extend(&self.block_header_hashes[i]);
        }

        bytes_vector.extend_from_slice(&self.stopping_hash);
        bytes_vector
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError> {
        if slice.len() < MINIMUM_ANOMUNT_OF_BYTES {
            return Err(MessageError::ErrorCreatingGetBlockHeadersMessage);
        }

        //checkear el tamaño max de la tira de bytes.
        match Self::_from_bytes(slice) {
            Some(get_header_message) => Ok(get_header_message),
            None => Err(MessageError::ErrorCreatingGetBlockHeadersMessage),
        }
    }

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("getheaders\0\0", &self.to_bytes())
    }
}

impl GetBlockHeadersMessage {
    /// Returns an instance of a GetBlockHeadersMessage
    pub fn new(
        version: u32,
        block_header_hashes: Vec<[u8; 32]>,
        stopping_hash: [u8; 32],
    ) -> GetBlockHeadersMessage {
        let hash_count = VarLenInt::new(block_header_hashes.len());
        GetBlockHeadersMessage {
            version,
            hash_count,
            block_header_hashes,
            stopping_hash,
        }
    }

    /// Receives a slice of bytes and returns a GetBlockHeadersMessage.
    /// If anything fails, None is returned.
    fn _from_bytes(slice: &[u8]) -> Option<GetBlockHeadersMessage> {
        let hash_count = VarLenInt::from_bytes(&slice[4..])?;

        if (hash_count.amount_of_bytes() + 32 + 4 + 32 * hash_count.to_usize()) != slice.len() {
            return None;
        }

        let version = u32::from_le_bytes(slice[0..4].try_into().ok()?);

        let mut used_bytes = 4 + hash_count.amount_of_bytes();
        let mut block_header_hashes: Vec<[u8; 32]> = Vec::new();
        while used_bytes < slice.len() - 32 {
            let hash: [u8; 32] = slice[used_bytes..(used_bytes + 32)].try_into().ok()?;
            block_header_hashes.push(hash);
            used_bytes += 32;
        }

        let stopping_hash = slice[used_bytes..].try_into().ok()?;

        Some(GetBlockHeadersMessage {
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
    use crate::utils::mock_tcp_stream::MockTcpStream;
    use bitcoin_hashes::{sha256d, Hash};

    // Auxiliar functions
    //=================================================================

    fn get_block_headers_message_expected_bytes() -> Vec<u8> {
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
    fn get_block_headers_message_test_1_to_bytes() -> Result<(), MessageError> {
        let expected_bytes = get_block_headers_message_expected_bytes();

        let mut vec_hash = Vec::new();
        let hash: [u8; 32] = *sha256d::Hash::hash(b"test").as_byte_array();
        vec_hash.push(hash);

        let get_block_headers_msg = GetBlockHeadersMessage::new(70015, vec_hash, hash);

        let get_block_headers_msg = get_block_headers_msg.to_bytes();

        assert_eq!(get_block_headers_msg, expected_bytes);
        Ok(())
    }

    #[test]
    fn get_block_headers_message_test_2_send_to() -> Result<(), MessageError> {
        let mut stream = MockTcpStream::new();

        let mut vec_hash = Vec::new();
        let hash: [u8; 32] = *sha256d::Hash::hash(b"test").as_byte_array();
        vec_hash.push(hash);

        let get_block_headers_msg = GetBlockHeadersMessage::new(70015, vec_hash, hash);
        let get_block_headers_hm = get_block_headers_msg.get_header_message()?;
        let mut expected_result = get_block_headers_hm.to_bytes();
        expected_result.extend(get_block_headers_msg.to_bytes());

        get_block_headers_msg.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }

    #[test]
    fn get_block_headers_message_test_3_from_bytes() -> Result<(), MessageError> {
        let mut vec_hash = Vec::new();
        let hash: [u8; 32] = *sha256d::Hash::hash(b"test").as_byte_array();
        vec_hash.push(hash);

        let expected_get_block_headers_msg = GetBlockHeadersMessage::new(70015, vec_hash, hash);

        let get_block_headers_msg = GetBlockHeadersMessage::from_bytes(
            &mut expected_get_block_headers_msg.to_bytes().as_mut_slice(),
        )?;

        assert_eq!(get_block_headers_msg, expected_get_block_headers_msg);
        Ok(())
    }
}
