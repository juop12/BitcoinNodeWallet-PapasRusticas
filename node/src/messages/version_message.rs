use super::message_trait::*;
use crate::utils::variable_length_integer::VarLenInt;
use chrono::Utc;
use rand::prelude::*;
use std::net::{IpAddr, SocketAddr};

const NODE_NETWORK: u64 = 0x01;
const MINIMAL_VERSION_MESSAGE_SIZE: usize = 86;

/// Contains all necessary fields, for sending a version message needed for doing a handshake among nodes
#[derive(Debug, PartialEq)]
pub struct VersionMessage {
    version: i32,
    services: u64,
    timestamp: i64,
    addr_recv_services: u64,
    receiver_address: [u8; 16],
    receiver_port: u16,
    addr_sender_services: u64,
    sender_address: [u8; 16],
    sender_port: u16,
    nonce: u64,
    user_agent_length: VarLenInt,
    user_agent: Vec<u8>,
    start_height: i32,
    relay: u8,
}

impl Message for VersionMessage {
    type MessageType = VersionMessage;
    const SENDING_ERROR: MessageError = MessageError::ErrorSendingVersionMessage;

    /// Implementation of the trait to_bytes for VersionMessage, returns a vector of bytes
    /// with all the attributes of the struct. Both little and big endian are used, following
    /// the BTC protocol
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        bytes_vector.extend_from_slice(&self.services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes_vector.extend_from_slice(&self.addr_recv_services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.receiver_address);
        bytes_vector.extend_from_slice(&self.receiver_port.to_be_bytes());
        bytes_vector.extend_from_slice(&self.addr_sender_services.to_le_bytes());
        bytes_vector.extend_from_slice(&self.sender_address);
        bytes_vector.extend_from_slice(&self.sender_port.to_be_bytes());
        bytes_vector.extend_from_slice(&self.nonce.to_le_bytes());
        bytes_vector.extend(&self.user_agent_length.to_bytes());
        if self.user_agent_length.to_usize() > 0 {
            bytes_vector.extend(&self.user_agent);
        }
        bytes_vector.extend_from_slice(&self.start_height.to_le_bytes());
        bytes_vector.push(self.relay);
        bytes_vector
    }

    /// Functions as a wrapper of _from for VersionMessage, recieves a slice of bytes and
    /// returns a Result with either a VersionMessage if everything went Ok or a MessageError
    /// if the call to _from failed. The slice must be at least 86 bytes long (the minimum
    /// length of a VersionMessage)
    fn from_bytes(slice: &[u8]) -> Result<Self::MessageType, MessageError> {
        if slice.len() < MINIMAL_VERSION_MESSAGE_SIZE {
            return Err(MessageError::ErrorCreatingVersionMessage);
        }

        match Self::_from_bytes(slice) {
            Some(version_msg) => Ok(version_msg),
            None => Err(MessageError::ErrorCreatingVersionMessage),
        }
    }

    /// Returns a HeaderMessage with the command "version" and the payload of the VersionMessage
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new("version\0\0\0\0\0", &self.to_bytes())
    }
}

impl VersionMessage {
    /// Constructor for the struct VersionMessage, receives a version and a receiver address (which
    /// includes both the ip and port) and returns an instance of a VersionMessage with all its
    /// necesary attributes initialized, the optional ones are left in blank
    pub fn new(
        version: i32,
        receiver_address: SocketAddr,
        sender_address: SocketAddr,
    ) -> Result<VersionMessage, MessageError> {
        let user_agent_length = VarLenInt::new(0);
        let version_msg = VersionMessage {
            version,
            services: NODE_NETWORK,
            timestamp: Utc::now().timestamp(),
            addr_recv_services: 0, //Como no sabemos que servicios admite el nodo asumimos que no admite ningun servicio
            receiver_address: {
                match receiver_address.ip() {
                    IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped().octets(),
                    IpAddr::V6(ipv6) => ipv6.octets(),
                }
            },
            receiver_port: receiver_address.port(),
            addr_sender_services: NODE_NETWORK,
            sender_address: {
                match sender_address.ip() {
                    IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped().octets(),
                    IpAddr::V6(ipv6) => ipv6.octets(),
                }
            },
            sender_port: sender_address.port(),
            nonce: rand::thread_rng().gen(),
            user_agent_length,
            user_agent: Vec::new(),
            start_height: 0,
            relay: 0x01,
        };

        Ok(version_msg)
    }

    /// Implementation of the trait _from for VersionMessage. Recieves a slice of bytes and
    /// returns an Option with either a VersionMessage if everything went Ok or None if any step
    /// in the middle of the conversion from bytes to VersionMessage fields failed.
    fn _from_bytes(slice: &[u8]) -> Option<VersionMessage> {
        if slice[80..].is_empty() {
            return None;
        }
        let user_agent_length = VarLenInt::from_bytes(&slice[80..]);
        let amount_of_bytes = user_agent_length.amount_of_bytes();

        let version_msg = VersionMessage {
            version: i32::from_le_bytes(slice[0..4].try_into().ok()?),
            services: u64::from_le_bytes(slice[4..12].try_into().ok()?),
            timestamp: i64::from_le_bytes(slice[12..20].try_into().ok()?),
            addr_recv_services: u64::from_le_bytes(slice[20..28].try_into().ok()?),
            receiver_address: slice[28..44].try_into().ok()?,
            receiver_port: u16::from_be_bytes(slice[44..46].try_into().ok()?),
            addr_sender_services: u64::from_le_bytes(slice[46..54].try_into().ok()?),
            sender_address: slice[54..70].try_into().ok()?,
            sender_port: u16::from_be_bytes(slice[70..72].try_into().ok()?),
            nonce: u64::from_le_bytes(slice[72..80].try_into().ok()?),
            user_agent_length,
            user_agent: Vec::from(&slice[(80 + amount_of_bytes)..(slice.len() - 5)]),
            start_height: i32::from_le_bytes(
                slice[(slice.len() - 5)..(slice.len() - 1)]
                    .try_into()
                    .ok()?,
            ),
            relay: slice[slice.len() - 1],
        };
        Some(version_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock_tcp_stream::MockTcpStream;
    use std::net::Ipv4Addr;

    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001;

    // Auxiliar functions
    //=================================================================

    fn create_socket() -> (SocketAddr, SocketAddr) {
        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
        let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));

        (receiver_socket, sender_socket)
    }

    fn version_message_without_user_agent_expected_bytes(timestamp: i64, rand: u64) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&timestamp.to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes());
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets());
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(0 as u8);
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }

    fn version_message_with_user_agent_expected_bytes() -> Vec<u8> {
        let rand: u64 = rand::thread_rng().gen();
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&(Utc::now().timestamp() as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes());
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets());
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(253 as u8);
        bytes_vector.extend_from_slice(&(4 as u16).to_le_bytes());
        bytes_vector.extend_from_slice(&"test".as_bytes());
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }

    // Tests
    //=================================================================

    #[test]
    fn version_message_test_1_to_bytes_without_user_agent() -> Result<(), MessageError> {
        let (receiver_socket, sender_socket) = create_socket();

        let version_msg = VersionMessage::new(70015, receiver_socket, sender_socket)?;

        let version_msg_bytes = version_msg.to_bytes();

        assert_eq!(
            version_msg_bytes,
            version_message_without_user_agent_expected_bytes(
                version_msg.timestamp,
                version_msg.nonce
            )
        );

        Ok(())
    }

    #[test]
    fn version_message_test_2_to_bytes_with_user_agent() -> Result<(), MessageError> {
        let mut expected_bytes = version_message_with_user_agent_expected_bytes();
        let version_msg = VersionMessage::from_bytes(&mut expected_bytes.as_mut_slice())?;

        let version_msg_bytes = version_msg.to_bytes();

        assert_eq!(version_msg_bytes, expected_bytes);
        Ok(())
    }

    #[test]
    fn version_message_test_3_send_to() -> Result<(), MessageError> {
        let mut stream = MockTcpStream::new();

        let (receiver_socket, sender_socket) = create_socket();

        let version_msg = VersionMessage::new(70015, receiver_socket, sender_socket)?;
        let hm = version_msg.get_header_message()?;
        let mut expected_result = hm.to_bytes();
        expected_result.extend(version_msg.to_bytes());

        version_msg.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    }

    #[test]
    fn version_message_test_4_from_bytes_without_user_agent() -> Result<(), MessageError> {
        let (receiver_socket, sender_socket) = create_socket();

        let expected_version_msg = VersionMessage::new(70015, receiver_socket, sender_socket)?;

        let version_msg =
            VersionMessage::from_bytes(&mut expected_version_msg.to_bytes().as_mut_slice())?;

        assert_eq!(version_msg, expected_version_msg);
        Ok(())
    }

    #[test]
    fn version_message_test_5_from_bytes_with_user_agent() -> Result<(), MessageError> {
        let (receiver_socket, sender_socket) = create_socket();

        let mut expected_version_msg = VersionMessage::new(70015, receiver_socket, sender_socket)?;
        expected_version_msg.user_agent_length = VarLenInt::from_bytes(&[253, 4, 0]);
        expected_version_msg.user_agent = Vec::from("test");

        let version_msg =
            VersionMessage::from_bytes(&mut expected_version_msg.to_bytes().as_mut_slice())?;

        assert_eq!(version_msg, expected_version_msg);
        Ok(())
    }
}
