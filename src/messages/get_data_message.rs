use super::utils::*;
use crate::messages::*;
use bitcoin_hashes::{sha256d, Hash};
// use std::{
//     io::{Read, Write},
// };

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
        todo!()
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
        let (count, amount_of_bytes, value) = calculate_variable_length_integer(&slice);
        let mut inventory: Vec<[u8;32]> = Vec::new();
        let mut i = amount_of_bytes;
        while i < slice.len(){
            let aux: [u8;32] = match slice[(i)..(i + 32)].try_into(){
                Ok(array) => array,
                Err(_) => return Err(MessageError::ErrorCreatingGetData),
            };
            inventory.push(aux);
            i += 32;
        }

        Ok(GetDataMessage::new(inventory,count))
    }

    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        todo!()
    }
}

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

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_to_bytes_get_data() -> Result<(), MessageError> {
            
        let (mut expected_bytes, hash1, hash2) = get_data_message_expected_bytes(false);
        let mut hashes  = vec![hash1, hash2];
        
        let block_headers_message = GetDataMessage::new(hashes,vec![2]);

        assert_eq!(block_headers_message.to_bytes(), expected_bytes);
        Ok(())
    } 
}