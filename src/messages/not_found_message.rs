use super::message_trait::*;
pub use crate::messages::inv_message::*;

/// Represents the NotFoundMessage as an InvMessage because it is equally implemented
pub struct NotFoundMessage {
    inv: InvMessage,
}

impl Message for NotFoundMessage {
    type MessageType = NotFoundMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingNotFoundMessage;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        self.inv.to_bytes()
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError> {
        let inv = match InvMessage::from_bytes(slice) {
            Ok(inv_message) => inv_message,
            Err(_) => return Err(MessageError::ErrorCreatingNotFoundMessage),
        };
        Ok(NotFoundMessage { inv })
    }

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("notfound\0\0\0\0", &self.to_bytes())
    }
}
