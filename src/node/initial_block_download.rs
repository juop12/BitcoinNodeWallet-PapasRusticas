use crate::node::*;
use bitcoin_hashes::{sha256d, Hash};


const HASHEDGENESISBLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68,
    0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
    0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1,
    0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
];// 0x64 | [u8; 32] 


impl Node {

    fn create_get_block_header_message(&self, hash: [u8; 32]) -> GetBlockHeadersMessage {
        let mut block_header_hashes = Vec::new();
        block_header_hashes.push(hash);
        let version = self.version as u32;
        let stopping_hash = [0_u8; 32];
    
        GetBlockHeadersMessage::new(version, block_header_hashes, stopping_hash)
    }
    
    fn ibd_receive_headers_message<T: Read + Write> (&self, mut stream: T) -> Result<BlockHeadersMessage, NodeError>{
        let block_headers_msg_h = self.receive_message_header(&mut stream)?;
        
        let mut msg_bytes = vec![0; block_headers_msg_h.get_payload_size() as usize];
        match stream.read_exact(&mut msg_bytes) {
            Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
            Ok(_) => {}
        }

        if block_headers_msg_h.get_command_name() != "headers\0\0\0\0\0" {
            return Err(NodeError::ErrorReceivingHeadersMessageHeaderInIBD);
        }else{
            println!("\n\n\nRECIBIMOS LOS HEADERS AAAAAAAAAAAAAAAAAAAA {:?}\n\n\n", block_headers_msg_h.get_command_name());
        }


        let block_headers_msg = match BlockHeadersMessage::from_bytes(&mut msg_bytes){
            Ok(block_headers_message) => block_headers_message,
            Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
        };
        println!("la cantidad de bloques es {:?}", block_headers_msg.count);
        Ok(block_headers_msg)
    }

    fn ibd_send_get_block_headers_message<T: Read + Write>(
        &self,
        last_hash: [u8; 32],
        stream: &mut T,
    ) -> Result<(), NodeError> {

        let get_block_headers_msg = self.create_get_block_header_message(last_hash);
        
        match get_block_headers_msg.send_to(stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInIBD),
        }
    }

    pub fn initial_block_download<T: Read + Write>(&self, tcp: T) -> Result<(), NodeError> {

        let mut sync_node =  tcp;//&self.tcp_streams[6];
        let mut block_headers: Vec<BlockHeader> = Vec::new();
        let mut quantity_received = 2000;
        let mut last_hash = HASHEDGENESISBLOCK;

        while quantity_received == 2000{
            
            self.ibd_send_get_block_headers_message(last_hash, &mut sync_node)?;

            let block_headers_msg = match self.ibd_receive_headers_message(&mut sync_node,){
                Ok(mensaje) => mensaje,
                Err(_) => continue, 
            };
            let received_block_headers = block_headers_msg.headers;        
    
            quantity_received = received_block_headers.len();
            last_hash = *sha256d::Hash::hash(&received_block_headers[quantity_received-1].to_bytes()).as_byte_array();
            
            block_headers.extend(received_block_headers);

        }
        Ok(())
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


    #[test]
    fn ibd_test_1_send_get_block_headers_message() -> Result<(), NodeError>{
        let mut stream = MockTcpStream::new();
        let node = Node::_new(VERSION, LOCAL_HOST, LOCAL_PORT);

        let expected_msg = node.create_get_block_header_message(HASHEDGENESISBLOCK);
        let expected_hm = expected_msg.get_header_message().unwrap();
        let mut expected_bytes = expected_hm.to_bytes();
        expected_bytes.extend(expected_msg.to_bytes());

        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, &mut stream)?;
        
        assert_eq!(stream.write_buffer, expected_bytes);
        Ok(())
    }

    #[test]
    fn ibd_test_2_receive_block_headers() -> Result<(), NodeError> {
        let mut stream = MockTcpStream::new();
        
        let node = Node::_new(VERSION, LOCAL_HOST, LOCAL_PORT);

        let hash1 :[u8;32] = *sha256d::Hash::hash(b"test1").as_byte_array();
        let hash2 :[u8;32] = *sha256d::Hash::hash(b"test2").as_byte_array();
        let merkle_hash1 :[u8;32] = *sha256d::Hash::hash(b"test merkle root1").as_byte_array();
        let merkle_hash2 :[u8;32] = *sha256d::Hash::hash(b"test merkle root2").as_byte_array();
        
        let b_h1 = BlockHeader::new(70015, hash1, merkle_hash1); 
        let b_h2 = BlockHeader::new(70015, hash2, merkle_hash2);
        
        let expected_msg = BlockHeadersMessage::new(vec![b_h1, b_h2]);
        let expected_hm = expected_msg.get_header_message().unwrap();
        stream.read_buffer = expected_hm.to_bytes();
        stream.read_buffer.extend(expected_msg.to_bytes());
        
        let received_message = node.ibd_receive_headers_message(stream)?;
        
        assert_eq!(received_message.to_bytes(), expected_msg.to_bytes());
        Ok(())
    }
}