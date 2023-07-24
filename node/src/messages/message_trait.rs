pub use super::*;
use super::{BlockMessage, InvMessage};
pub use crate::utils::btc_errors::MessageError;
pub use std::io::{Read, Write};

/// All messages that can be sent or received by a node in the bitcoin network must implement this trait.
pub trait MessageTrait {
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

pub enum Message {
    BlockHeaders(BlockHeadersMessage),
    Block(BlockMessage),
    GetBlockHeaders(GetBlockHeadersMessage),
    GetData(GetDataMessage),
    Header(HeaderMessage),
    Inv(InvMessage),
    NotFound(NotFoundMessage),
    Tx(TxMessage),
    VerACK(VerACKMessage),
    Version(VersionMessage),
    Ping(PingMessage),
    Pong(PongMessage),
    UnknownMessage,
}

impl Message {
    pub fn from_bytes(bytes: Vec<u8>, command_name: String) -> Result<Message, MessageError> {
        let mensaje = match command_name.as_str() {
            "headers\0\0\0\0\0" => Message::BlockHeaders(BlockHeadersMessage::from_bytes(&bytes)?),
            "block\0\0\0\0\0\0\0" => Message::Block(BlockMessage::from_bytes(&bytes)?),
            "getheaders\0\0" => {
                Message::GetBlockHeaders(GetBlockHeadersMessage::from_bytes(&bytes)?)
            }
            "getdata\0\0\0\0\0" => Message::GetData(GetDataMessage::from_bytes(&bytes)?),
            "header\0\0\0\0\0\0" => Message::Header(HeaderMessage::from_bytes(&bytes)?),
            "inv\0\0\0\0\0\0\0\0\0" => Message::Inv(InvMessage::from_bytes(&bytes)?),
            "notfound\0\0\0\0" => Message::NotFound(NotFoundMessage::from_bytes(&bytes)?),
            "tx\0\0\0\0\0\0\0\0\0\0" => Message::Tx(TxMessage::from_bytes(&bytes)?),
            "verack\0\0\0\0\0\0" => Message::VerACK(VerACKMessage::from_bytes(&bytes)?),
            "version\0\0\0\0\0" => Message::Version(VersionMessage::from_bytes(&bytes)?),
            "ping\0\0\0\0\0\0\0\0" => Message::Ping(PingMessage::from_bytes(&bytes)?),
            "pong\0\0\0\0\0\0\0\0" => Message::Pong(PongMessage::from_bytes(&bytes)?),
            _ => Message::UnknownMessage,
        };
        Ok(mensaje)
    }
}
