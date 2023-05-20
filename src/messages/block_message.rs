use super::utils::*;
use crate::messages::*;
use crate::blocks::blockchain::Block;
use crate::blocks::transaction::Transaction;

pub struct BlockMessage{
    pub block :Block,
}

impl Message for BlockMessage {
    type MessageType = BlockMessage;
    /// Writes the message as bytes in the receiver_stream
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>{
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorSendingGetDataMessage),
       }
    }

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        self.block.to_bytes()
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError>{
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

impl BlockMessage{
    /*pub fn block(&self)-> Block{
        self.block
    }*/
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::utils::mock_tcp_stream::MockTcpStream;
    use bitcoin_hashes::{sha256d, Hash};
    
    fn block_expected_bytes()->Vec<u8>{
        let mut expected_bytes =  block_header_expected_bytes();
        expected_bytes.push(2);
        //temporal hasta que definiamos que son las transacciones
        let transaction = Transaction::new(70015, Vec::new(), Vec::new(), 0);
        expected_bytes
    }

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
