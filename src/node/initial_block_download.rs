use crate::node::*;
use bitcoin_hashes::{sha256d, Hash};
use std::{io::{BufRead, BufReader}, fs::File, path::Path, char::MAX};

const HASHEDGENESISBLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68,
    0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
    0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1,
    0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
];// 0x64 | [u8; 32] 

const BLOCK_IDENTIFIER: [u8; 4] = [0x02, 0x00, 0x00, 0x00];
const MAX_RECEIVED_HEADERS: usize = 2000;

impl Node {

    fn create_get_block_header_message(&self, hash: [u8; 32]) -> GetBlockHeadersMessage {
        let mut block_header_hashes = Vec::new();
        block_header_hashes.push(hash);
        let version = self.version as u32;
        let stopping_hash = [0_u8; 32];

        GetBlockHeadersMessage::new(version, block_header_hashes, stopping_hash)
    }

    ///Creates and sends a GetBlockHeadersMessage to the stream, always asking for the maximum amount of headers. On error returns ErrorSendingMessageInIBD
    fn ibd_send_get_block_headers_message(
        &self,
        last_hash: [u8; 32],
        sync_node_index: usize,
    ) -> Result<(), NodeError> {

        let get_block_headers_msg = self.create_get_block_header_message(last_hash);
        println!("Mandamos {:?}", get_block_headers_msg);
        let mut stream = &self.tcp_streams[sync_node_index];
        
        match get_block_headers_msg.send_to(&mut stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInIBD),
        }
    }

    ///Handles the headers message by hashing the last received header and asking for more headers
    fn handle_block_headers_message(&mut self, mut msg_bytes :Vec<u8>, sync_node_index: usize)-> Result<(), NodeError>{
        let block_headers_msg = match BlockHeadersMessage::from_bytes(&mut msg_bytes){
            Ok(block_headers_message) => block_headers_message,
            Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
        };
        println!("Recibimos {:?} headers", block_headers_msg.count);
        let received_block_headers = block_headers_msg.headers;
        let quantity_received = received_block_headers.len();

        let last_header_hash = *sha256d::Hash::hash(&received_block_headers[quantity_received -1].to_bytes()).as_byte_array();
        
        self.block_headers.extend(received_block_headers);
        if quantity_received == MAX_RECEIVED_HEADERS{
            self.ibd_send_get_block_headers_message(last_header_hash, sync_node_index)?;
        }
        Ok(())
    }
    
    ///Generic receive message function, receives a header and its payload, and calls the corresponding handler. Returns the command name in the received header
    fn receive_message (&mut self, sync_node_index: usize) -> Result<String, NodeError>{
        let mut stream = &self.tcp_streams[sync_node_index];
        let block_headers_msg_h = self.receive_message_header(&mut stream)?;
        println!("\n\n{}", block_headers_msg_h.get_command_name());
        
        let mut msg_bytes = vec![0; block_headers_msg_h.get_payload_size() as usize];
        match stream.read_exact(&mut msg_bytes) {
            Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
            Ok(_) => {}
        }

        match block_headers_msg_h.get_command_name().as_str(){
            "headers\0\0\0\0\0" => self.handle_block_headers_message(msg_bytes, sync_node_index)?,
            "block\0\0\0\0\0\0\0" => self.handle_block_message(msg_bytes)?,
            _ => {},
        }

        Ok(block_headers_msg_h.get_command_name())

    }

    //works for <253 hashes
    fn send_get_data_message_for_block(&self, hashes :Vec<[u8; 32]>, sync_node_index: usize)->Result<(), NodeError>{
        let count = vec![hashes.len() as u8];
        let mut inventory = Vec::new();
        for _ in 0..hashes.len(){
            
            inventory.push(GetDataMessage::as_block_element(inventory_element));
        }
        
        let get_data_message = GetDataMessage::new(inventory, count);
        
        let mut stream = &self.tcp_streams[sync_node_index];
        
        match get_data_message.send_to(&mut stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInIBD),
        }
    }

    fn handle_block_message(&mut self, mut msg_bytes :Vec<u8>)-> Result<(), NodeError>{

        let block_msg = match BlockMessage::from_bytes(&mut msg_bytes){
            Ok(block_message) => block_message,
            Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
        };
        //let blocks_file = Self::_open_blocks_handler("blocks.csv");
        self.blockchain.push(block_msg.block);
        Ok(())
    }

    fn _open_blocks_handler(path: &str) -> Result<File, ConfigError> {
        match File::open(path){
            Ok(file)=> Ok(file),
            Err(_) => Err(ConfigError::ErrorReadingFile),
        }
    }

    pub fn initial_block_download(&mut self) -> Result<(), NodeError> {
        let sync_node_index :usize = 2;
        let last_hash = HASHEDGENESISBLOCK;
        let mut headers_received = self.block_headers.len();
        
        self.ibd_send_get_block_headers_message(last_hash, sync_node_index)?;
        
        while headers_received == self.block_headers.len(){
            if self.receive_message(sync_node_index)? == "headers\0\0\0\0\0"{
                headers_received = self.block_headers.len();
            }
            //validar que el header del bloque diga que es de la fecha de la consigna en adelante (modificar config)
            //descargar bloques validos (aplicar concurrencia)

            println!("# de headers = {headers_received}");
        }
        let blocks_to_download: [u8;32] = match self.block_headers[0].to_bytes().try_into(){
            Ok(hash) => hash,
            Err(_) => return Err(NodeError::ErrorSendingMessageInIBD),
        };
        match self.send_get_data_message_for_block(vec![blocks_to_download], sync_node_index){
            Ok(_) => {},
            Err(_) => return Err(NodeError::ErrorSendingMessageInIBD),
        }
        self.receive_message(sync_node_index)?;

        
        println!("# de headers = {headers_received}");
        println!("# de headers = {}", self.block_headers.len());
        Ok(())
        
    }
}

#[cfg(test)]

mod tests{
    use super::*;

    #[test]
    fn test_IBD_1_can_download_blocks(){
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/node_log.txt"),
        };
        let mut node = Node::new(config);
        assert!(node.initial_block_download().is_ok());
        assert_eq!(node.blockchain.len(), 1);
    }
}