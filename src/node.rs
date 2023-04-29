use std::{ net::{SocketAddr, ToSocketAddrs, TcpStream}, io::{Read,Write} };
use proyecto::messages::*;

// use messages::VersionMessage;

const DNS_PORT: u16 = 53; //DNS PORT
const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
const LOCAL_PORT: u16 = 1001;
const MESAGE_HEADER_SIZE: usize = 24;

/// Struct that represents the errors that can occur in the Node
#[derive(Debug)]
enum NodeError{
    ErrorConnectingToPeer,
    ErrorSendingVersionMessageInHandshake,
    ErrorSendingVerAckMessageInHandshake,
    ErrorReceivingVersionMessageInHandshake,
    ErrorReceivingVerAckMessageInHandshake,
}

/// Struct that represents the bitcoin node
struct Node {
    version: i32,
    sender_address: SocketAddr,
}

impl Node {
    pub fn new() -> Node {
        Node { 
            version: 70015,
            sender_address: SocketAddr::from((LOCAL_HOST, LOCAL_PORT)),
        }
    }
    /// Receives a dns address as a String and returns a Vector that contains all the addresses
    /// returned by the dns. If an error occured (for example, the dns address is not valid), it
    /// returns an empty Vector.
    /// The socket address requires a dns and a DNS_PORT, which is set to 53 by default
    fn peer_discovery(&self, dns: String) -> Vec<SocketAddr> {
        let mut socket_address_vector = Vec::new();
        match (dns, DNS_PORT).to_socket_addrs() {
            Ok(address_iter) => {
                for address in address_iter {
                    socket_address_vector.push(address);
                }
            }
            Err(_) => {}
        }
        socket_address_vector
    }
    
    fn connect_to_peer(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError>{
        match TcpStream::connect(receiving_addrs){
            Ok(tcp_stream) => Ok(tcp_stream),
            Err(_) => Err(NodeError::ErrorConnectingToPeer),
        }
    }

    fn handshake_send_version_message<T: Read + Write>(&self,receiving_addrs: SocketAddr, mut stream : T) -> Result<VersionMessage, NodeError>{
        let vm = match VersionMessage::new(self.version, receiving_addrs, self.sender_address){
            Ok(version_message) => version_message,
            Err(_) => return Err(NodeError::ErrorSendingVersionMessageInHandshake),
        };
        match vm.send_to(&mut stream){
            Ok(_) => Ok(vm),
            Err(_) => Err(NodeError::ErrorSendingVersionMessageInHandshake),
        }
    }

    fn handshake_receive_version_message<T: Read + Write>(&self, mut stream :T) -> Result<(HeaderMessage, VersionMessage), NodeError>{
        let mut header_bytes = [0;MESAGE_HEADER_SIZE];
        match stream.read(&mut header_bytes) {
            Ok(_) => {},
            Err(_) => return Err(NodeError::ErrorReceivingVersionMessageInHandshake),
        }
        let received_vm_header = match HeaderMessage::from_bytes(&mut header_bytes) {
            Ok(header_message) => header_message,
            Err(_) => return Err(NodeError::ErrorReceivingVersionMessageInHandshake),
        };
        
        //armar vm recibido
        let mut received_vm_bytes = Vec::with_capacity(received_vm_header.get_payload_size() as usize);
        match stream.read(&mut received_vm_bytes){
            Ok(_) => {},
            Err(_) => return Err(NodeError::ErrorReceivingVersionMessageInHandshake),
        };
        match VersionMessage::from_bytes(&mut received_vm_bytes){
            Ok(version_message) => Ok((received_vm_header, version_message)),
            Err(_) => Err(NodeError::ErrorReceivingVersionMessageInHandshake),
        }
        
        //guardar datos que haga falta del vm
    }

    fn handshake_send_verack_message<T: Read + Write>(&self, mut stream : T) -> Result<VerACKMessage, NodeError>{
        let verack = match VerACKMessage::new(){
            Ok(version_message) => version_message,
            Err(_) => return Err(NodeError::ErrorSendingVerAckMessageInHandshake),
        };
        match verack.send_to(&mut stream){
            Ok(_) => Ok(verack),
            Err(_) => return Err(NodeError::ErrorSendingVerAckMessageInHandshake),
        }
    }

    fn handshake_receive_verack_message<T: Read + Write>(&self, mut stream :T) -> Result<HeaderMessage, NodeError>{
        let mut header_bytes = [0;MESAGE_HEADER_SIZE];
        match stream.read(&mut header_bytes) {
            Ok(_) => {},
            Err(_) => return Err(NodeError::ErrorReceivingVerAckMessageInHandshake),
        }
        let received_verack_header = match HeaderMessage::from_bytes(&mut header_bytes) {
            Ok(header_message) => header_message,
            Err(_) => return Err(NodeError::ErrorReceivingVerAckMessageInHandshake),
        };
        if received_verack_header.get_payload_size() == 0 &&  received_verack_header.get_command_name() == "verack\0\0\0\0\0\0".as_bytes(){
            return Ok(received_verack_header);
        }
        Err(NodeError::ErrorSendingVerAckMessageInHandshake)
    }

    fn handshake(&self, receiving_addrs: SocketAddr) -> Result<TcpStream, NodeError>{
        let mut tcp_stream = self.connect_to_peer(receiving_addrs)?;
        //enviar versionmessage
        self.handshake_send_version_message(receiving_addrs, &tcp_stream)?;
        //recibir version_message
        self.handshake_receive_version_message(&tcp_stream)?;
        //mandar verack
        self.handshake_send_verack_message(&tcp_stream)?;
        //recibimos verack
        self.handshake_receive_verack_message(&tcp_stream)?;
        Ok(tcp_stream)
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self};
    use bitcoin_hashes::{sha256d, Hash};

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

    /// Has both read and write buffers to test if the messages are correctly sent
    struct MockTcpStream {
        read_buffer: Vec<u8>,
        write_buffer: Vec<u8>,
    }

    impl MockTcpStream {
        /// Constructor for MockTcpStream
        fn new() -> MockTcpStream {
            MockTcpStream {
                read_buffer: Vec::new(),
                write_buffer: Vec::new(),
            }
        }
    }

    impl Read for MockTcpStream {
        /// Reads bytes from the stream until completing the buffer and returns how many bytes were read
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.read_buffer.as_slice().read(buf)
        }
    }

    impl Write for MockTcpStream {
        /// Writes the buffer value on the stream and returns how many bytes were written
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.write_buffer.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.write_buffer.flush()
        }
    }
    
    #[test]
    fn test_hanshake_1_send_version_message() -> Result<(), NodeError>{
        let node = Node::new();
        let mut stream = MockTcpStream::new();
        let receiver_socket = SocketAddr::from(([127,0,0,2], 8080));
        let expected_vm = VersionMessage::new(node.version, receiver_socket ,node.sender_address).unwrap();
        
        node.handshake_send_version_message(receiver_socket, &mut stream)?;
        let write_buffer_len = stream.write_buffer.len();

        //reemplaza el valor erroneo de checksum por el esperado debido al diferente random
        let expected_hm = expected_vm.get_header_message().unwrap();
        let mut hm_expected_bytes = expected_hm.to_bytes();
        let hm_size = hm_expected_bytes.len();
        let hash = sha256d::Hash::hash(&stream.write_buffer[MESAGE_HEADER_SIZE..(write_buffer_len)]);
        let hash_value = hash.as_byte_array();
        for i in 0..4{
            hm_expected_bytes[hm_size -i -1] = hash_value[3-i];
        }
        
        assert_eq!(stream.write_buffer[0..MESAGE_HEADER_SIZE], hm_expected_bytes);

        let mut vm_expected_bytes = expected_vm.to_bytes();
        for i in 0..8{
            vm_expected_bytes [72+i] = stream.write_buffer[72+MESAGE_HEADER_SIZE+i];
        }

        assert_eq!(stream.write_buffer[24..write_buffer_len], vm_expected_bytes);
        
        Ok(())
    }

    #[test]
    fn test_hanshake_2_receive_version_message() -> Result<(), NodeError>{
        let node = Node::new();
        let mut stream = MockTcpStream::new();
        let receiver_socket = SocketAddr::from(([127,0,0,2], 8080));
        let expected_vm = VersionMessage::new(node.version, receiver_socket ,node.sender_address).unwrap();
        stream.read_buffer = expected_vm.to_bytes();

        node.handshake_receive_verack_message(&mut stream)?;
        let read_buffer_len = stream.read_buffer.len();
        Ok(())
    }
}
