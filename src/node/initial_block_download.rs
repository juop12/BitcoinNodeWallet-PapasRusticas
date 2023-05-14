use crate::node::*;
use bitcoin_hashes::{sha256d, Hash};
use std::{io::{BufRead, BufReader}, fs::File, path::Path, char::MAX};
use chrono::{DateTime, TimeZone,Utc};
use block_downloader::*;
use std::thread;
use std::time::Duration;


const HASHEDGENESISBLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68,
    0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
    0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1,
    0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
];// 0x64 | [u8; 32] 
const BLOCK_IDENTIFIER: [u8; 4] = [0x02, 0x00, 0x00, 0x00];
const MAX_RECEIVED_HEADERS: usize = 2000;
const MAX_BLOCK_BUNDLE: usize = 16;
const STARTING_BLOCK_TIME: u32 = 1681084800; // https://www.epochconverter.com/, 2023-04-10 00:00:00 GMT


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
        
        self.block_headers.extend(received_block_headers);
        Ok(())
    }
    
    ///Generic receive message function, receives a header and its payload, and calls the corresponding handler. Returns the command name in the received header
    fn receive_message (&mut self, sync_node_index: usize) -> Result<String, NodeError>{
        let mut stream = &self.tcp_streams[sync_node_index];
        let block_headers_msg_h = receive_message_header(&mut stream)?;
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
    fn send_get_data_message_for_blocks(&self, hashes :Vec<[u8; 32]>, sync_node_index: usize)->Result<(), NodeError>{
        let count = vec![hashes.len() as u8];
        
        let get_data_message = GetDataMessage::create_message_inventory_block_type(hashes, count);
        println!("{:?}", get_data_message);
        
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

    fn get_block_bundle(&mut self, requested_block_hashes_bundled: Vec<[u8;32]>)-> Result<(), NodeError>{
        println!("\n\nentre a block bundle");
        let amount_of_hashes = requested_block_hashes_bundled.len();
        self.send_get_data_message_for_blocks(requested_block_hashes_bundled, 0)?;
        for _ in 0..amount_of_hashes{
            let mut received_message_type = self.receive_message(0)?;
            println!("no es el primer receive");
            while (received_message_type != "block\0\0\0\0\0\0\0") && (received_message_type != "notfound\0\0\0\0"){
                println!("sigo aca");
                received_message_type = self.receive_message(0)?;
            }
        }
        println!("Tengo # de blockes = {}", self.blockchain.len());
        Ok(())
    }

    pub fn initial_block_download(&mut self) -> Result<(), NodeError> {
        let sync_node_index :usize = 0;
        let mut last_hash = HASHEDGENESISBLOCK;
        let mut headers_received = self.block_headers.len();
        let mut request_block_hashes_bundle :Vec<[u8;32]> = Vec::new();
        
        //self.ibd_send_get_block_headers_message(last_hash, sync_node_index)?;
        
        while headers_received == self.block_headers.len(){
            self.ibd_send_get_block_headers_message(last_hash, sync_node_index)?;
            while(self.receive_message(sync_node_index)? != "headers\0\0\0\0\0"){

            }
            println!("recibi el message header");
            let mut i = headers_received;
            headers_received += 2000;
            last_hash = self.block_headers[self.block_headers.len()-1].hash();

            if self.block_headers[i].time() > STARTING_BLOCK_TIME{
                while i < self.block_headers.len(){
                    if self.block_headers[i].time() > STARTING_BLOCK_TIME { 
                        println!("meti un hash en el bundle");
                        request_block_hashes_bundle.push(self.block_headers[i].hash());
                        if request_block_hashes_bundle.len() == MAX_BLOCK_BUNDLE{
                            self.get_block_bundle(request_block_hashes_bundle);
                            request_block_hashes_bundle = Vec::new();
                        }
                    }
                    i+= 1;
                }
            }
            println!("# de headers = {headers_received}");
        }

        if request_block_hashes_bundle.len() >0{
            self.get_block_bundle(request_block_hashes_bundle);
        }
        
        println!("# de headers = {headers_received}");
        println!("# de headers = {}", self.block_headers.len());
        Ok(())
        
    }
}

#[cfg(test)]

mod tests{
    use super::*;
    use std::{
        sync::{Arc, Mutex},
    };

    //test unitario de descargaqr un solo header
    
    /*
    #[test]
    fn ibd_test_1_can_download_blocks() -> Result<(), NodeError>{
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/node_log.txt"),
            begin_time: 1681084800,
        };
        let sync_node_index = 0;
        let mut node = Node::new(config);
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        while node.receive_message(sync_node_index)? != "headers\0\0\0\0\0" {

        }

        let mut block_hashes_bundle = Vec::new();
        for i in 0..16{
            block_hashes_bundle.push(node.block_headers[i].hash());
        }
        node.get_block_bundle(block_hashes_bundle)?;
        
        assert!(node.blockchain.len() > 1);
        Ok(())
        //node.receive_message(sync_node_index);
    }
    */

    #[test]
    fn ibd_test_2_can_download_blocks() -> Result<(), NodeError>{
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/node_log.txt"),
            begin_time: 1681084800,
        };
        let sync_node_index = 0;
        let mut node = Node::new(config);
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        while node.receive_message(sync_node_index)? != "headers\0\0\0\0\0" {

        }
        let vec = Vec::new();
        let safe_block_chain = Arc::new(Mutex::from(vec));
        let block_downloader = BlockDownloader::new(node.get_tcp_streams(), &safe_block_chain).unwrap();

        for j in 0..125{
            let mut block_hashes_bundle :Vec<[u8;32]> = Vec::new();
            for i in 0..16{
                block_hashes_bundle.push(node.block_headers[j*16 + i].hash());
            }
            block_downloader.download_block_bundle(block_hashes_bundle).unwrap();
        }

        block_downloader.finish_downloading();
        //thread::sleep(Duration::from_secs(30));
        
        let a = safe_block_chain.lock().unwrap();
        assert!(a.len() >= 1);
        Ok(())
        //node.receive_message(sync_node_index);
    }
    
    
}