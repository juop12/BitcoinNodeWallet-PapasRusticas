use std::net::TcpStream;

use super::message_trait::*;

/// Represents the NotFoundMessage as an InvMessage because it is equally implemented
pub struct PingMessage {
    nonce: [u8; 8],
}

impl MessageTrait for PingMessage {
    type MessageType = PingMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingNotFoundMessage;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        Vec::from(self.nonce)
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError> {
        let nonce: [u8; 8] = slice[0..8]
            .try_into()
            .map_err(|_| MessageError::ErrorCreatingPingMessage)?;
        Ok(PingMessage { nonce })
    }

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("ping\0\0\0\0\0\0\0\0", &self.to_bytes())
    }
}

impl PingMessage {
    pub fn reply_pong(&self, stream: &mut TcpStream) -> Result<(), MessageError> {
        PongMessage::from(self.nonce).send_to(stream)
    }
}

pub struct PongMessage {
    nonce: [u8; 8],
}

impl MessageTrait for PongMessage {
    type MessageType = PongMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingPongMessage;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        Vec::from(self.nonce)
    }

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError> {
        let nonce: [u8; 8] = slice[0..8]
            .try_into()
            .map_err(|_| MessageError::ErrorCreatingPongMessage)?;
        Ok(PongMessage { nonce })
    }

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("pong\0\0\0\0\0\0\0\0", &self.to_bytes())
    }
}

impl PongMessage {
    fn from(nonce: [u8; 8]) -> PongMessage {
        PongMessage { nonce }
    }
}
