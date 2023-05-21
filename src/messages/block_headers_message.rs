use super::utils::*;
use crate::blocks::blockchain::BlockHeader;
use crate::utils::variable_length_integer::*;

const BLOCKHEADER_SIZE: usize = 80;
const BLOCKHEADERS_MSG_NAME: &str = "headers\0\0\0\0\0";


/// The BlockHeader struct represents a block header in the Bitcoin network.
#[derive(Debug, PartialEq)]
pub struct BlockHeadersMessage {
    pub count: VarLenInt,
    pub headers: Vec<BlockHeader>,
}

impl Message for BlockHeadersMessage{

    type MessageType = BlockHeadersMessage;
    
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>{
        todo!()
    }
    
    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        let mut bytes_vector = self.count.to_bytes();
        for header in &self.headers{
            bytes_vector.extend(header.to_bytes());
            bytes_vector.push(0);
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
        HeaderMessage::new("headers\0\0\0\0\0", &self.to_bytes())
    }
} 

impl BlockHeadersMessage{

    pub fn new(headers: Vec<BlockHeader>, count: VarLenInt) -> BlockHeadersMessage{
    //     let mut count = Vec::new();
    //     count.push(headers.len() as u8); //estamos asumiendo que solo van de 253 a menor
        BlockHeadersMessage{
            count,
            headers,
        }
    }

    fn _from_bytes(slice: &mut [u8]) -> Option<BlockHeadersMessage> {
        let count = VarLenInt::from_bytes(slice);
        
        if (count.to_usize() * 81 + count.amount_of_bytes()) != slice.len(){
            return None;
        }
        
        let mut headers: Vec<BlockHeader> = Vec::new();
        let first_header_position = count.amount_of_bytes();

        let mut i = count.amount_of_bytes();
        while i < slice.len(){
            let mut block_headers_bytes = Vec::from(&slice[(i)..(i + 80)]);
            let bloc_header = BlockHeader::from_bytes(&mut block_headers_bytes).ok()?;
            headers.push(bloc_header);
            i += 81;
        }
        Some(BlockHeadersMessage::new(headers, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_hashes::{sha256d, Hash};


    // Auxiliar functions
    //=================================================================

    fn block_headers_message_expected_bytes(double_bytes_for_count :bool) -> (Vec<u8>, BlockHeader, BlockHeader){
        let hash1 :[u8;32] = *sha256d::Hash::hash(b"test1").as_byte_array();
        let hash2 :[u8;32] = *sha256d::Hash::hash(b"test2").as_byte_array();
        let merkle_hash1 :[u8;32] = *sha256d::Hash::hash(b"test merkle root1").as_byte_array();
        let merkle_hash2 :[u8;32] = *sha256d::Hash::hash(b"test merkle root2").as_byte_array();
        
        let b_h1 = BlockHeader::new(70015, hash1, merkle_hash1, 0); 
        let b_h2 = BlockHeader::new(70015, hash2, merkle_hash2, 0);

        let mut expected_bytes = Vec::new();
        if double_bytes_for_count{
            expected_bytes.push(253);
            expected_bytes.extend_from_slice(&(2 as u16).to_le_bytes());
        }else{
            expected_bytes.push(2);

        }
            
        expected_bytes.extend(b_h1.to_bytes());
        expected_bytes.push(0);
        expected_bytes.extend(b_h2.to_bytes());
        expected_bytes.push(0);
        
        (expected_bytes, b_h1, b_h2)
    }

    // Test
    //=================================================================

    #[test]
    fn block_headers_message_test_1_to_bytes() -> Result<(), MessageError> {
        
        let (expected_bytes, b_h1, b_h2) = block_headers_message_expected_bytes(false);
        let mut block_headers = Vec::new();
        block_headers.push(b_h1);
        block_headers.push(b_h2);

        let count = VarLenInt::from_bytes(&expected_bytes);
        let block_headers_message = BlockHeadersMessage::new(block_headers,count);

        assert_eq!(block_headers_message.to_bytes(), expected_bytes);
        Ok(())
    }

    #[test]
    fn block_headers_message_test_2_from_bytes () -> Result<(), MessageError> {
        let (mut expected_bytes, b_h1, b_h2) = block_headers_message_expected_bytes(false);
        let mut block_headers = Vec::new();
        block_headers.push(b_h1);
        block_headers.push(b_h2);

        let count = VarLenInt::from_bytes(&expected_bytes);
        let expected_block_headers_message = BlockHeadersMessage::new(block_headers, count);

        let block_headers_message = BlockHeadersMessage::from_bytes(&mut expected_bytes)?;
        assert_eq!(block_headers_message, expected_block_headers_message);
        Ok(())

    }

    #[test]
    fn block_headers_message_test_3_from_bytes_with_more_than_one_byte_in_coun() -> Result<(), MessageError> {
        let (mut expected_bytes, _b_h1, _b_h2) = block_headers_message_expected_bytes(true);

        let block_headers_message = BlockHeadersMessage::from_bytes(&mut expected_bytes)?;
        assert_eq!(block_headers_message.to_bytes(), expected_bytes);
        Ok(())

    }
}
