
use super::message_trait::*;
pub use crate::messages::inv_message::*;

/// Represents the GetDataMessage as an InvMessage because it is equally implemented
pub struct GetDataMessage{
    inv: InvMessage
}

impl GetDataMessage{
    /*
    /// Creates an instance of a 
    pub fn new(inventory: Vec<[u8;36]>) -> GetDataMessage{
        GetDataMessage{inv: InvMessage::new(inventory)}
    }*/

    pub fn create_message_inventory_block_type(inventory_entries: Vec<[u8;32]>) -> GetDataMessage{
        GetDataMessage{inv: InvMessage::create_message_inventory_block_type(inventory_entries)}
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
        self.inv.to_bytes()
    }

    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError>{
        let inv = match InvMessage::from_bytes(slice){
            Ok(inv_message) => inv_message,
            Err(_) => return Err(MessageError::ErrorCreatingGetDataMessage)
        };
        Ok(GetDataMessage{inv})
    }

    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        HeaderMessage::new("getdata\0\0\0\0\0", &self.to_bytes())
    }
}