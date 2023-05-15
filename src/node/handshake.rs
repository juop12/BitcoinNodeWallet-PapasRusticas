use crate::node::*;

impl Node {

    ///Returns a tcp stream representing the conection with the peer, if this fails returns ErrorConnectingToPeer
    fn connect_to_peer(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError> {
        match TcpStream::connect(receiving_addrs) {
            Ok(tcp_stream) => Ok(tcp_stream),
            Err(_) => Err(NodeError::ErrorConnectingToPeer),
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
        let hm = receive_message_header(&mut stream)?;

        if hm.get_command_name() != "version\0\0\0\0\0" {
            return Err(NodeError::ErrorReceivingMessageInHandshake);
        }

        let mut received_vm_bytes = vec![0; hm.get_payload_size() as usize];

        match stream.read_exact(&mut received_vm_bytes) {
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
        let hm = receive_message_header(stream)?;

        if hm.get_payload_size() == 0 && hm.get_command_name() == "verack\0\0\0\0\0\0" {
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
    pub fn handshake(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError> {
        let tcp_stream = self.connect_to_peer(receiving_addrs)?;

        self.handshake_send_version_message(receiving_addrs, &tcp_stream)?;

        self.handshake_receive_version_message(&tcp_stream)?;

        self.handshake_send_verack_message(&tcp_stream)?;

        self.handshake_receive_verack_message(&tcp_stream)?;

        Ok(tcp_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_tcp_stream::*;
    use bitcoin_hashes::{sha256d, Hash};


    const VERSION: i32 = 70015;
    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001; 


    // Auxiliar functions
    //=================================================================

    fn initiate() -> (MockTcpStream, Node) {
        let stream = MockTcpStream::new();
        let logger = Logger::from_path("test_log.txt").unwrap();
        let node = Node::_new(VERSION, LOCAL_HOST, LOCAL_PORT, logger);

        (stream, node)
    }

    // Tests
    //=================================================================

    #[test]
    fn handshake_test_1_send_version_message() -> Result<(), NodeError> {
        let (mut stream, node) = initiate();

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
        let (mut stream, node) = initiate(); 

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
    fn handshake_test_3_send_verack_message() -> Result<(), NodeError> {
        let (mut stream, node) = initiate();

        let expected_verack_msg = VerACKMessage::new().unwrap();
        let verack_hm = expected_verack_msg.get_header_message().unwrap();
        let expected_bytes = verack_hm.to_bytes();

        node.handshake_send_verack_message(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_bytes);
        Ok(())
    }

    #[test]
    fn handshake_test_4_receive_verack_message() -> Result<(), NodeError> {
        let (mut stream, node) = initiate();

        let expected_verack_msg = VerACKMessage::new().unwrap();
        let verack_hm = expected_verack_msg.get_header_message().unwrap();
        stream.read_buffer = verack_hm.to_bytes();

        let received_verack_msg = node.handshake_receive_verack_message(&mut stream)?;
        assert_eq!(received_verack_msg, expected_verack_msg);
        Ok(())
    }
}