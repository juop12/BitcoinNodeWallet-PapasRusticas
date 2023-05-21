pub use crate::messages::inv_message::*;
use super::utils::*;


const NOTFOUND_MSG_NAME: &str = "notfound\0\0\0\0";


/// Represents the NotFoundMessage as an InvMessage because it is equally implemented
pub struct NotFoundMessage{
    inv: InvMessage
}

impl Message for NotFoundMessage{
    type MessageType = NotFoundMessage;
    
    //Writes the message as bytes in the receiver_stream
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>{
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorsendingNotFoundMessage),
       }
    }

    //transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>{
        self.inv.to_bytes()
    }

    //Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError>{
        let inv = match InvMessage::from_bytes(slice){
            Ok(inv_message) => inv_message,
            Err(_) => return Err(MessageError::ErrorCreatingNotFoundMessage)
        };
        Ok(NotFoundMessage{inv})
    }

    //Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>{
        HeaderMessage::new(NOTFOUND_MSG_NAME, &self.to_bytes())
    }
}