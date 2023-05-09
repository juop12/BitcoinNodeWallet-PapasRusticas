pub use std::io::{Read, Write};
pub use super::HeaderMessage;

/// Error Struct for messages, contains customized errors for each type of message (excluding
/// VerACKMessage) and to diferenciate whether the error occured while instanciation or in
/// message sending
#[derive(Debug, PartialEq)]
pub enum MessageError {
    ErrorCreatingVersionMessage,
    ErrorSendingVersionMessage,
    ErrorCreatingHeaderMessage,
    ErrorSendingHeaderMessage,
    ErrorCreatingVerAckMessage,
    ErrorSendingVerAckMessage,
    ErrorCreatingGetBlockHeadersMessage,
    ErrorSendingGetBlockHeadersMessage,
    ErrorCreatingBlockHeadersMessage,
    ErrorHeadersBlockMessage,
}

//Hacer un wrapper para send to,cosa de que solo se pueda mandar un tcpStream?
pub trait Message {
    type MessageType;
    /// Writes the message as bytes in the receiver_stream
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError>;

    /// Transforms the message to bytes, usig the p2p bitcoin protocol
    fn to_bytes(&self) -> Vec<u8>;

    /// Creates the coresponding message, using a slice of bytes, wich must be of the correct size, otherwise an error will be returned.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError>;

    /// Gets the header message corresponding to the corresponding message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError>;
}

/// Gets the variable lenght based on the bytes of the slice. The size depends on what the
/// position 0 of the slice is, based on that it can be 1, 3, 5 or 9 bytes long.
/// Returns a tuple with the variable length and the amount of bytes that it has
pub fn calculate_variable_length_integer(slice: &[u8]) -> (Vec<u8>, usize) {
    let mut length = Vec::new();
    let mut amount_of_bytes = 1;
    if slice[0] == 0xfd {
        amount_of_bytes = 3;
    }
    if slice[0] == 0xfe {
        amount_of_bytes = 5;
    }
    if slice[0] == 0xff {
        amount_of_bytes = 9;
    }
    for i in slice.iter().take(amount_of_bytes) {
        length.push(*i);
    }
    (length, amount_of_bytes)
}