use super::utils::*;
use crate::messages::*;

#[derive(Debug)]
pub struct GetDataMessage {
    count: Vec<u8>,
    inventory: Vec<[u8;32]>,
}
impl GetDataMessage{
    pub fn new(inventory: Vec<[u8;32]>, count: Vec<u8>) -> GetDataMessage{
        GetDataMessage{
                count,
                inventory,
            }
        }
}

impl Message for GetDataMessage{
    type MessageType = GetDataMessage;
    //Writes the message as bytes in the receiver_stream
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>{
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorSendingGetDataMessage),
       }
    }

    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        let mut bytes_vector = Vec::new();
        bytes_vector.extend(&self.count);
       
        for element in &self.inventory {
            bytes_vector.extend(element);
        }
        bytes_vector
    }

    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError>{
        let (count, amount_of_bytes, inventory_size) = calculate_variable_length_integer(&slice);

        if (inventory_size * 32 + amount_of_bytes) != slice.len(){
            return Err(MessageError::ErrorCreatingGetDataMessage)
        }

        let mut inventory: Vec<[u8;32]> = Vec::new();
        let mut i = amount_of_bytes;
        while i < slice.len(){
            let aux: [u8;32] = match slice[(i)..(i + 32)].try_into(){
                Ok(array) => array,
                Err(_) => return Err(MessageError::ErrorCreatingGetDataMessage),
            };
            inventory.push(aux);
            i += 32;
        }

        Ok(GetDataMessage::new(inventory,count))
    }

    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        HeaderMessage::new("getdata\0\0\0\0\0", &self.to_bytes())
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::mock_tcp_stream::MockTcpStream;
    use bitcoin_hashes::{sha256d, Hash};

    fn get_data_message_expected_bytes(double_bytes_for_count :bool) -> (Vec<u8>, [u8;32], [u8;32]){
        let hash1 :[u8;32] = *sha256d::Hash::hash(b"test1").as_byte_array();
        let hash2 :[u8;32] = *sha256d::Hash::hash(b"test2").as_byte_array();
    
        let mut expected_bytes = Vec::new();
        if double_bytes_for_count{
            expected_bytes.push(253);
            expected_bytes.extend_from_slice(&(2 as u16).to_le_bytes());
        }else{
            expected_bytes.push(2);
        }
        expected_bytes.extend(hash1);
        expected_bytes.extend(hash2);
        (expected_bytes, hash1, hash2)
    }

    #[test]
    fn get_data_test1_to_bytes() -> Result<(), MessageError> {
            
        let (expected_bytes, hash1, hash2) = get_data_message_expected_bytes(false);
        let hashes  = vec![hash1, hash2];
        
        let block_headers_message = GetDataMessage::new(hashes,vec![2]);

        assert_eq!(block_headers_message.to_bytes(), expected_bytes);
        Ok(())
    } 

    #[test]
    fn get_data_test2_cant_create_from_an_incorrect_ammount_of_bytes(){
        let (mut expected_bytes, hash1, hash2) = get_data_message_expected_bytes(false);
        expected_bytes.push(0);

        GetDataMessage::from_bytes(&mut expected_bytes).unwrap_err();
    }

    #[test]
    fn get_data_test3_message_is_created_properly_from_correct_amount_of_bytes()->Result<(), MessageError>{
        let (mut expected_bytes, hash1, hash2) = get_data_message_expected_bytes(false);
        let get_data_message = GetDataMessage::from_bytes(&mut expected_bytes)?;

        assert_eq!(get_data_message.to_bytes(), expected_bytes);
        Ok(())
    }

    #[test]
    fn get_block_headers_message_test_4_send_to()-> Result<(), MessageError> {
        let mut stream = MockTcpStream::new();
        let (mut message_bytes, hash1, hash2) = get_data_message_expected_bytes(false);
        let get_data_message = GetDataMessage::from_bytes(&mut message_bytes)?;

        let get_data_hm = get_data_message.get_header_message()?;
        let mut expected_result = get_data_hm.to_bytes();
        expected_result.extend(message_bytes);

        get_data_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }
}