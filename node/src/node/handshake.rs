use std::time::Duration;

use crate::node::*;

pub const PEER_TIMEOUT: Duration = Duration::from_secs(15);
impl Node {
    /// Returns a tcp stream representing the conection with the peer, if this fails returns ErrorConnectingToPeer
    fn connect_to_peer(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError> {
        match TcpStream::connect_timeout(&receiving_addrs, PEER_TIMEOUT) {
            Ok(tcp_stream) => {
                tcp_stream.set_write_timeout(Some(PEER_TIMEOUT)).map_err(|_| NodeError::ErrorConnectingToPeer)?;
                Ok(tcp_stream)
            },
            Err(_) => Err(NodeError::ErrorConnectingToPeer),
        }
    }

    /// Sends the version message as bytes to the stream according to bitcoin protocol. On error returns ErrorSendingMessageInHandshake
    pub fn handshake_send_version_message<T: Read + Write>(
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

    /// Sends the verack message to the stream according to bitcoin protocol. On error returns ErrorSendingMessageInHandshake
    pub fn handshake_send_verack_message<T: Read + Write>(
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

    /// Receives a message, if it is any other than VersionMessage or VerackMessage it returns ErrorReceivingMessageInHandshake
    fn handshake_receive_verack_or_version_message<T: Read + Write>(
        &self,
        mut stream: T,
    ) -> Result<String, NodeError> {
        let hm = receive_message_header(&mut stream)?;

        let mut received_vm_bytes = vec![0; hm.get_payload_size() as usize];
        match stream.read_exact(&mut received_vm_bytes) {
            Ok(_) => {}
            Err(_) => return Err(NodeError::ErrorReceivingMessageInHandshake),
        };

        self.logger.log(format!("Received message: {}", hm.get_command_name()));
        let cmd_name = hm.get_command_name();

        match cmd_name.as_str() {
            "version\0\0\0\0\0" => match VersionMessage::from_bytes(&received_vm_bytes) {
                Ok(_) => Ok(cmd_name),
                Err(_) => Err(NodeError::ErrorReceivingMessageInHandshake),
            },
            "verack\0\0\0\0\0\0" => {
                if hm.get_payload_size() == 0 {
                    Ok(cmd_name)
                } else {
                    Err(NodeError::ErrorReceivingMessageInHandshake)
                }
            }
            _ => Err(NodeError::ErrorReceivingMessageInHandshake),
        }
    }

    /// Does peer conection protocol acording to the bitcoin network. Sends a VersionMessage,
    /// and it receives a VersionMessage and a VerAckMessage, not in any order on particular.
    /// If everything works well returns a tcpstream, which lets us communicate with the peer._err
    pub fn handshake(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError> {
        let tcp_stream = self.connect_to_peer(receiving_addrs)?;

        self.handshake_send_version_message(receiving_addrs, &tcp_stream)?;

        let first_msg_name = self.handshake_receive_verack_or_version_message(&tcp_stream)?;
        let second_msg_name = self.handshake_receive_verack_or_version_message(&tcp_stream)?;

        if first_msg_name == second_msg_name {
            return Err(NodeError::ErrorReceivingMessageInHandshake);
        }

        self.handshake_send_verack_message(&tcp_stream)?;

        Ok(tcp_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock_tcp_stream::*;
    use bitcoin_hashes::{sha256d, Hash};


    const VERSION: i32 = 70015;
    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001;
    const STARTING_BLOCK_TIME: u32 = 1681084800;
    const HEADERS_FILE_PATH: &str = "data/headers.bin";
    const BLOCKS_FILE_PATH: &str = "data/blocks.bin";


    // Auxiliar functions
    //=================================================================

    fn initiate(log_file_path: &str) -> (MockTcpStream, Node) {
        let stream = MockTcpStream::new();
        let logger = Logger::from_path(log_file_path).unwrap();
        let data_handler = NodeDataHandler::new(HEADERS_FILE_PATH, BLOCKS_FILE_PATH).unwrap();
        let node = Node::_new(
            VERSION,
            LOCAL_HOST,
            LOCAL_PORT,
            logger,
            data_handler,
            STARTING_BLOCK_TIME,
        );

        (stream, node)
    }

    // Tests
    //=================================================================

    #[test]
    fn handshake_test_1_send_version_message() -> Result<(), NodeError> {
        let (mut stream, node) = initiate("tests_txt/test_log.txt");

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
            sha256d::Hash::hash(&stream.write_buffer[MESSAGE_HEADER_SIZE..(write_buffer_len)]);
        let hash_value = hash.as_byte_array();

        for i in 0..4 {
            hm_expected_bytes[hm_size - i - 1] = hash_value[3 - i];
        }

        assert_eq!(
            stream.write_buffer[0..MESSAGE_HEADER_SIZE],
            hm_expected_bytes
        );

        let mut vm_expected_bytes = expected_vm.to_bytes();
        for i in 0..8 {
            vm_expected_bytes[72 + i] = stream.write_buffer[72 + MESSAGE_HEADER_SIZE + i];
        }

        assert_eq!(stream.write_buffer[24..write_buffer_len], vm_expected_bytes);

        Ok(())
    }

    #[test]
    fn handshake_test_2_receive_version_message() -> Result<(), NodeError> {
        let (mut stream, node) = initiate("tests_txt/handshake_test_2_log.txt");

        let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
        let expected_vm =
            VersionMessage::new(node.version, receiver_socket, node.sender_address).unwrap();
        let expected_hm = expected_vm.get_header_message().unwrap();
        stream.read_buffer = expected_hm.to_bytes();
        stream.read_buffer.extend(expected_vm.to_bytes());

        let received_mg = node.handshake_receive_verack_or_version_message(&mut stream)?;

        assert_eq!(received_mg, "version\0\0\0\0\0");
        Ok(())
    }

    #[test]
    fn handshake_test_3_send_verack_message() -> Result<(), NodeError> {
        let (mut stream, node) = initiate("tests_txt/test_log.txt");

        let expected_verack_msg = VerACKMessage::new().unwrap();
        let verack_hm = expected_verack_msg.get_header_message().unwrap();
        let expected_bytes = verack_hm.to_bytes();

        node.handshake_send_verack_message(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_bytes);
        Ok(())
    }

    #[test]
    fn handshake_test_4_receive_verack_message() -> Result<(), NodeError> {
        let (mut stream, node) = initiate("tests_txt/handshake_test_4_log.txt");

        let expected_verack_msg = VerACKMessage::new().unwrap();
        let verack_hm = expected_verack_msg.get_header_message().unwrap();
        stream.read_buffer = verack_hm.to_bytes();

        let received_msg = node.handshake_receive_verack_or_version_message(&mut stream)?;
        assert_eq!(received_msg, "verack\0\0\0\0\0\0");
        Ok(())
    }
}
