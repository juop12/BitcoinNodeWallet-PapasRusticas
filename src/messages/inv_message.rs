use super::message_trait::*;
use crate::utils::variable_length_integer::VarLenInt;

const BLOCK_IDENTIFIER: [u8; 4] = [0x02, 0x00, 0x00, 0x00];

#[derive(Debug)]
struct Entry{
    inv_type: [u8;4],
    hash: [u8;32],
}

#[derive(Debug)]
pub struct InvMessage {
    count: VarLenInt,
    inventory: Vec<Entry>,
}

impl InvMessage{
    fn new(inventory: Vec<Entry>) -> InvMessage{
        InvMessage{
                count: VarLenInt::new(inventory.len()),
                inventory,
        }
    }

    pub fn create_message_inventory_block_type(inventory_entries: Vec<[u8;32]>) -> InvMessage{
        let mut inventory: Vec<Entry> = Vec::new();
        for hash in inventory_entries{
            inventory.push(Entry::as_block_entry(hash))
        };
        Self::new(inventory)
    }
}

impl Message for InvMessage{
    type MessageType = InvMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingInvMessage;
        

    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        let mut bytes_vector = Vec::new();
        bytes_vector.extend(&self.count.to_bytes());
       
        for entry in &self.inventory {
            bytes_vector.extend(entry.to_bytes());
        }
        bytes_vector
    }

    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError>{
        let count = VarLenInt::from_bytes(&slice);

        if (count.to_usize() * 36 + count.amount_of_bytes()) != slice.len(){
            return Err(MessageError::ErrorCreatingInvMessage)
        }

        let mut inventory: Vec<Entry> = Vec::new();
        let mut i = count.amount_of_bytes();
        while i < slice.len(){
            let aux: [u8;36] = match slice[(i)..(i + 36)].try_into(){
                Ok(array) => array,
                Err(_) => return Err(MessageError::ErrorCreatingInvMessage),
            };
            inventory.push(Entry::from_bytes(aux)?);
            i += 36;
        }

        Ok(InvMessage{
            inventory,
            count,
        })
    }

    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        HeaderMessage::new("inv\0\0\0\0\0\0\0\0\0", &self.to_bytes())
    }
}

impl InvMessage{
    pub fn get_block_hashes(&self)-> Vec<[u8;32]>{
        let mut block_hashes: Vec<[u8;32]> = Vec::new();
        for entry in &self.inventory{
            if entry.is_block_type(){
                block_hashes.push(entry.hash);
            }
        }
        block_hashes
    }
}

impl Entry{
    fn as_block_entry(hash: [u8;32]) -> Entry{
        Entry{ inv_type: BLOCK_IDENTIFIER, hash }
    }

    fn to_bytes(&self)-> Vec<u8>{
        let mut bytes = Vec::from(self.inv_type);
        bytes.extend(self.hash);
        bytes
    }

    fn from_bytes(bytes: [u8;36])-> Result<Entry, MessageError>{
        let inv_type: [u8;4] = match bytes[0..4].try_into(){
            Ok(array) => array,
            Err(_) => return Err(MessageError::ErrorCreatingInvMessage),
        };
        let hash: [u8;32] = match bytes[4..36].try_into(){
            Ok(array) => array,
            Err(_) => return Err(MessageError::ErrorCreatingInvMessage),
        };
        Ok(Entry{ inv_type, hash})
    }

    fn is_block_type(&self)-> bool{
        self.inv_type == BLOCK_IDENTIFIER
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::utils::mock_tcp_stream::MockTcpStream;
    use bitcoin_hashes::{sha256d, Hash};

    fn inv_message_expected_bytes(double_bytes_for_count :bool) -> (Vec<u8>, [u8;32], [u8;32]){
        let hash1 :[u8;32] = *sha256d::Hash::hash(b"test1").as_byte_array();
        let hash2 :[u8;32] = *sha256d::Hash::hash(b"test2").as_byte_array();
    
        let mut expected_bytes = Vec::new();
        if double_bytes_for_count{
            expected_bytes.push(253);
            expected_bytes.extend_from_slice(&(2 as u16).to_le_bytes());
        }else{
            expected_bytes.push(2);
        }
        expected_bytes.extend(Entry::as_block_entry(hash1).to_bytes());
        expected_bytes.extend(Entry::as_block_entry(hash2).to_bytes());
        (expected_bytes, hash1, hash2)
    }

    #[test]
    fn inv_test1_to_bytes() -> Result<(), MessageError> {
            
        let (expected_bytes, hash1, hash2) = inv_message_expected_bytes(false);
        
        let hashes  = vec![Entry::as_block_entry(hash1),Entry::as_block_entry(hash2)];
        
        let block_headers_message = InvMessage::new(hashes);

        assert_eq!(block_headers_message.to_bytes(), expected_bytes);
        Ok(())
    } 

    #[test]
    fn inv_test2_cant_create_from_an_incorrect_ammount_of_bytes(){
        let (mut expected_bytes, hash1, hash2) = inv_message_expected_bytes(false);
        expected_bytes.push(0);

        InvMessage::from_bytes(&mut expected_bytes).unwrap_err();
    }

    #[test]
    fn inv_test3_message_is_created_properly_from_correct_amount_of_bytes()->Result<(), MessageError>{
        let (mut expected_bytes, hash1, hash2) = inv_message_expected_bytes(false);
        let inv_message = InvMessage::from_bytes(&mut expected_bytes)?;

        assert_eq!(inv_message.to_bytes(), expected_bytes);
        Ok(())
    }

    #[test]
    fn get_block_headers_message_test_4_send_to()-> Result<(), MessageError> {
        let mut stream = MockTcpStream::new();
        let (mut message_bytes, hash1, hash2) = inv_message_expected_bytes(false);
        let inv_message = InvMessage::from_bytes(&mut message_bytes)?;

        let inv_hm = inv_message.get_header_message()?;
        let mut expected_result = inv_hm.to_bytes();
        expected_result.extend(message_bytes);

        inv_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }
}