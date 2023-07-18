pub use super::HeaderMessage;
pub use crate::utils::btc_errors::MessageError;
pub use std::io::{Read, Write};

/// All messages that can be sent or received by a node in the bitcoin network must implement this trait.
pub trait Message {
    type MessageType;
    const SENDING_ERROR: MessageError;

    /// Writes the message as bytes in the receiver_stream
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError> {
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write_all(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(Self::SENDING_ERROR),
        }
    }

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>;

    /// Creates the coresponding message, using a slice of bytes, wich must be
    /// of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError>;

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>;
}
