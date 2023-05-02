use proyecto::messages::*;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
};

// use messages::VersionMessage;
const VERSION: i32 = 70015;
const DNS_PORT: u16 = 53; //DNS PORT
const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 1001;
const MESAGE_HEADER_SIZE: usize = 24;

/// Struct that represents the errors that can occur in the Node
#[derive(Debug)]
enum NodeError {
    ErrorConnectingToPeer,
    ErrorSendingMessageInHandshake,
    ErrorReceivingMessageInHandshake,
}

/// Struct that represents the bitcoin node
struct Node {
    version: i32,
    sender_address: SocketAddr,
}

impl Node {
    pub fn new() -> Node {
        Node {
            version: VERSION,
            sender_address: SocketAddr::from((LOCAL_HOST, LOCAL_PORT)),
        }
    }
    /// Receives a dns address as a String and returns a Vector that contains all the addresses
    /// returned by the dns. If an error occured (for example, the dns address is not valid), it
    /// returns an empty Vector.
    /// The socket address requires a dns and a DNS_PORT, which is set to 53 by default
    fn peer_discovery(&self, dns: String) -> Vec<SocketAddr> {
        let mut socket_address_vector = Vec::new();
        if let Ok(address_iter) = (dns, DNS_PORT).to_socket_addrs() {
            for address in address_iter {
                socket_address_vector.push(address);
            }
        }
        socket_address_vector
    }

    ///Returns a tcp stream representing the conection with the peer, if this fails returns ErrorConnectingToPeer
    fn connect_to_peer(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError> {
        match TcpStream::connect(receiving_addrs) {
            Ok(tcp_stream) => Ok(tcp_stream),
            Err(_) => Err(NodeError::ErrorConnectingToPeer),
        }
    }

    ///Reads from the stream MESAGE_HEADER_SIZE and returns a HeaderMessage interpreting those bytes acording to bitcoin protocol.
    /// On error returns ErrorReceivingMessageInHandshake
    fn handshake_receive_header_message<T: Read + Write>(
        &self,
        mut stream: T,
    ) -> Result<HeaderMessage, NodeError> {
        let mut header_bytes = [0; MESAGE_HEADER_SIZE];
        match stream.read(&mut header_bytes) {
            Ok(_) => {}
            Err(_) => return Err(NodeError::ErrorReceivingMessageInHandshake),
        };

        match HeaderMessage::from_bytes(&mut header_bytes) {
            Ok(header_message) => Ok(header_message),
            Err(_) => Err(NodeError::ErrorReceivingMessageInHandshake),
        }
    }

    ///Sends the version message as bytes to the stream according to bitcoin protocol. On error returns ErrorSendingMessageInHandshake
    fn handshake_send_version_message<T: Read + Write>(
        &self,
        receiving_addrs: SocketAddr,
        mut stream: T,
    ) -> Result<(), NodeError> {
        let vm = match VersionMessage::new(self.version, receiving_addrs, self.sender_address) {
            Ok(version_message) => version_message,
            Err(_) => return Err(NodeError::ErrorSendingMessageInHandshake),
        };

        match vm.send_to(&mut stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInHandshake),
        }
    }

    ///Reads from the stream first a header message, and then reads as many bytes as tat header indicated. With this latter batch of
    ///bytes it tries to return a VersionMessage interpreting bytes according to bitcoin protocol. On error returns ErrorReceivingMessageInHandshake
    fn handshake_receive_version_message<T: Read + Write>(
        &self,
        mut stream: T,
    ) -> Result<VersionMessage, NodeError> {
        let hm = self.handshake_receive_header_message(&mut stream)?;

        let mut received_vm_bytes = vec![0; hm.get_payload_size() as usize];
        // read_exact???
        match stream.read(&mut received_vm_bytes) {
            Ok(_) => {}
            Err(_) => return Err(NodeError::ErrorReceivingMessageInHandshake),
        };

        match VersionMessage::from_bytes(&mut received_vm_bytes) {
            Ok(version_message) => Ok(version_message),
            Err(_) => Err(NodeError::ErrorReceivingMessageInHandshake),
        }
    }

    ///Sends the verack message to the stream according to bitcoin protocol. On error returns ErrorSendingMessageInHandshake
    fn handshake_send_verack_message<T: Read + Write>(
        &self,
        mut stream: T,
    ) -> Result<(), NodeError> {
        let verack = match VerACKMessage::new() {
            Ok(version_message) => version_message,
            Err(_) => return Err(NodeError::ErrorSendingMessageInHandshake),
        };

        match verack.send_to(&mut stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInHandshake),
        }
    }

    ///Reads a header from the stream, if that header represents a version_ack_message then returns VerACKMessage.
    /// On error returns ErrorSendingMessageInHandshake
    fn handshake_receive_verack_message<T: Read + Write>(
        &self,
        stream: T,
    ) -> Result<VerACKMessage, NodeError> {
        let hm = self.handshake_receive_header_message(stream)?;

        if hm.get_payload_size() == 0 && hm.get_command_name() == "verack\0\0\0\0\0\0".as_bytes() {
            //no se si falta hacer el segundo chequeo
            match VerACKMessage::new() {
                Ok(message) => return Ok(message),
                Err(_) => return Err(NodeError::ErrorSendingMessageInHandshake),
            }
        }

        Err(NodeError::ErrorSendingMessageInHandshake)
    }

    ///Does peer conection protocol acording to the bitcoin network. Sends a VersionMessage, recieves one, sends a VerACKMessage then receives one.
    ///If everything works returns a tcpstream,
    fn handshake(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError> {
        let tcp_stream = self.connect_to_peer(receiving_addrs)?;

        self.handshake_send_version_message(receiving_addrs, &tcp_stream)?;

        self.handshake_receive_version_message(&tcp_stream)?;
        //guardar datos que haga falta del vm

        self.handshake_send_verack_message(&tcp_stream)?;

        self.handshake_receive_verack_message(&tcp_stream)?;

        Ok(tcp_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_hashes::{sha256d, Hash};
    use proyecto::mock_tcp_stream::*;

    //test peer_discovery

    #[test]
    fn test_1_peer_discovery_fails_when_receiving_invalid_dns() {
        let node = Node::new();
        let address_vector = node.peer_discovery(String::from("does_not_exist"));
        assert!(address_vector.is_empty());
    }

    #[test]
    fn test_2_peer_discovery_returns_ip_vector_when_receiving_valid_dns() {
        let node = Node::new();
        let address_vector =
            node.peer_discovery(String::from("testnet-seed.bitcoin.jonasschnelli.ch"));
        assert!(!address_vector.is_empty());
    }

    //test handshake
    //para testear handshake es lo mismo que testear las funciones que lo conforman

    #[test]
    fn test_handshake_1_send_version_message() -> Result<(), NodeError> {
        let node = Node::new();
        let mut stream = MockTcpStream::new();
        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
        let expected_vm =
            VersionMessage::new(node.version, receiver_socket, node.sender_address).unwrap();

        node.handshake_send_version_message(receiver_socket, &mut stream)?;
        let write_buffer_len = stream.write_buffer.len();

        //reemplaza el valor erroneo de checksum por el esperado debido al diferente random
        let expected_hm = expected_vm.get_header_message().unwrap();
        let mut hm_expected_bytes = expected_hm.to_bytes();
        let hm_size = hm_expected_bytes.len();
        let hash =
            sha256d::Hash::hash(&stream.write_buffer[MESAGE_HEADER_SIZE..(write_buffer_len)]);
        let hash_value = hash.as_byte_array();
        for i in 0..4 {
            hm_expected_bytes[hm_size - i - 1] = hash_value[3 - i];
        }

        assert_eq!(
            stream.write_buffer[0..MESAGE_HEADER_SIZE],
            hm_expected_bytes
        );

        let mut vm_expected_bytes = expected_vm.to_bytes();
        for i in 0..8 {
            vm_expected_bytes[72 + i] = stream.write_buffer[72 + MESAGE_HEADER_SIZE + i];
        }

        assert_eq!(stream.write_buffer[24..write_buffer_len], vm_expected_bytes);

        Ok(())
    }

    #[test]
    fn test_handshake_2_receive_header_message() -> Result<(), NodeError> {
        let node = Node::new();
        let mut stream = MockTcpStream::new();
        let expected_hm =
            HeaderMessage::new("test message", &Vec::from("test".as_bytes())).unwrap();
        stream.read_buffer = expected_hm.to_bytes();

        let received_hm = node.handshake_receive_header_message(&mut stream)?;
        assert_eq!(received_hm, expected_hm);
        Ok(())
    }

    #[test]
    fn test_handshake_3_receive_version_message() -> Result<(), NodeError> {
        let node = Node::new();
        let mut stream = MockTcpStream::new();
        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
        let expected_vm =
            VersionMessage::new(node.version, receiver_socket, node.sender_address).unwrap();
        let expected_hm = expected_vm.get_header_message().unwrap();
        stream.read_buffer = expected_hm.to_bytes();
        stream.read_buffer.extend(expected_vm.to_bytes());

        let received_vm = node.handshake_receive_version_message(&mut stream)?;
        assert_eq!(received_vm.to_bytes(), expected_vm.to_bytes());
        Ok(())
    }

    #[test]
    fn test_handshake_4_send_verack_message() -> Result<(), NodeError> {
        let node = Node::new();
        let mut stream = MockTcpStream::new();
        let expected_ver_ack_message = VerACKMessage::new().unwrap();
        let ver_ack_header = expected_ver_ack_message.get_header_message().unwrap();
        let expected_bytes = ver_ack_header.to_bytes();

        node.handshake_send_verack_message(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_bytes);
        Ok(())
    }

    #[test]
    fn test_handshake_5_receive_verack_message() -> Result<(), NodeError> {
        let node = Node::new();
        let mut stream = MockTcpStream::new();
        let expected_ver_ack_message = VerACKMessage::new().unwrap();
        let ver_ack_header = expected_ver_ack_message.get_header_message().unwrap();
        stream.read_buffer = ver_ack_header.to_bytes();

        let received_ver_ack_message = node.handshake_receive_verack_message(&mut stream)?;
        assert_eq!(received_ver_ack_message, expected_ver_ack_message);
        Ok(())
    }
}
