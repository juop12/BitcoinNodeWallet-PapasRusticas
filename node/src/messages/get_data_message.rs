use super::message_trait::*;
pub use crate::messages::inv_message::*;

/// Represents the GetDataMessage as an InvMessage because it is equally implemented
pub struct GetDataMessage {
    inv: InvMessage,
}

impl Message for GetDataMessage {
    type MessageType = GetDataMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingGetDataMessage;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        self.inv.to_bytes()
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError> {
        let inv = match InvMessage::from_bytes(slice) {
            Ok(inv_message) => inv_message,
            Err(_) => return Err(MessageError::ErrorCreatingGetDataMessage),
        };
        Ok(GetDataMessage { inv })
    }

    /// Gets the header message corresponding to the corresponding message.
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("getdata\0\0\0\0\0", &self.to_bytes())
    }
}

impl GetDataMessage {
    /// Creates a new GetDataMessage with the given inventory entries interpreted as block hashes.
    pub fn create_message_inventory_block_type(inventory_entries: Vec<[u8; 32]>) -> GetDataMessage {
        GetDataMessage {
            inv: InvMessage::create_message_inventory_block_type(inventory_entries),
        }
    }

    pub fn create_message_inventory_transaction_type(
        inventory_entries: Vec<[u8; 32]>,
    ) -> GetDataMessage {
        GetDataMessage {
            inv: InvMessage::create_message_inventory_transaction_type(inventory_entries),
        }
    }

    pub fn get_block_hashes(&self) -> Vec<[u8; 32]>{
        self.inv.get_block_hashes()
    }
}
