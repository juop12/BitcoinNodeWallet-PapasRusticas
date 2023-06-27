use super::message_trait::*;
use crate::blocks::Transaction;
use crate::messages::*;

/// Struct that represents a block message.
pub struct TxMessage {
    pub tx: Transaction,
}

impl Message for TxMessage {
    type MessageType = TxMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingTxMessage;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        self.tx.to_bytes()
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError> {
        match Transaction::from_bytes(slice) {
            Ok(tx) => Ok(TxMessage { tx }),
            Err(_) => Err(MessageError::ErrorCreatingBlockMessage),
        }
    }

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("tx", &self.to_bytes())
    }
}

impl TxMessage {
    pub fn new(tx: Transaction) -> TxMessage {
        TxMessage { tx }
    }
}
