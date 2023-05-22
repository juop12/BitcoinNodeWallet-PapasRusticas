use crate::blocks::blockchain::Block;
use super::message_trait::*;
use crate::messages::*;


/// Struct that represents a block message.
pub struct BlockMessage{
    pub block: Block,
}

impl Message for BlockMessage {
    type MessageType = BlockMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingBlockMessage;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        self.block.to_bytes()
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError>{
        match Block::from_bytes(slice){
            Ok(block) => Ok(BlockMessage {block})  ,
            Err(_) => return Err(MessageError::ErrorCreatingBlockMessage)
        }
    }

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        HeaderMessage::new("block", &self.to_bytes())
    }
}


#[cfg(test)]
mod test{
    use super::*;
    use crate::utils::mock_tcp_stream::MockTcpStream;
    use crate::blocks::transaction::Transaction;
    use bitcoin_hashes::{sha256d, Hash};

    
    // Auxiliar functions
    //=================================================================

    fn block_header_expected_bytes() -> Vec<u8>{
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test merkle root").as_byte_array());
        bytes_vector.extend_from_slice(&(0 as u32).to_le_bytes());
        bytes_vector.extend_from_slice(&(0x30c31b18 as u32).to_be_bytes());
        bytes_vector.extend_from_slice(&(14082023 as u32).to_le_bytes());
        bytes_vector
    }

    fn block_expected_bytes()->Vec<u8>{
        let mut expected_bytes = block_header_expected_bytes();
        expected_bytes.push(2);

        let tx1 = Transaction::new(70015, Vec::new(), Vec::new(), 0);
        let tx2 = Transaction::new(70015, Vec::new(), Vec::new(), 0);

        expected_bytes.extend(tx1.to_bytes());
        expected_bytes.extend(tx2.to_bytes());
        
        expected_bytes
    }

    // Tests
    //=================================================================

    #[test]
    fn block_message_test_1_send_to()-> Result<(), MessageError> {
        let mut stream = MockTcpStream::new();
        let mut message_bytes = block_expected_bytes();
        let block_message = BlockMessage::from_bytes(&mut message_bytes)?;

        let block_hm = block_message.get_header_message()?;
        let mut expected_result = block_hm.to_bytes();
        expected_result.extend(message_bytes);

        block_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }
}
