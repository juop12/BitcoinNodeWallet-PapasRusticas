use crate::node::*;
use block_downloader::*;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant}
};


const HASHEDGENESISBLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68,
    0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
    0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1,
    0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
];// 0x64 | [u8; 32] 
const STARTING_BLOCK_TIME: u32 = 1681084800; // https://www.epochconverter.com/, 2023-04-10 00:00:00 GMT
const MAX_BLOCK_BUNDLE: usize = 16;
const HEADER_TIME_OUT: u64 = 30;


impl Node {
    ///Creates a GetBlockHeadersMessage with the given hash
    fn create_get_block_header_message(&self, hash: [u8; 32]) -> GetBlockHeadersMessage {
        let mut block_header_hashes = Vec::new();
        block_header_hashes.push(hash);
        let version = self.version as u32;
        let stopping_hash = [0_u8; 32];
      
        GetBlockHeadersMessage::new(version, block_header_hashes, stopping_hash)
    }

    ///Creates and sends a GetBlockHeadersMessage to the stream, always asking for the maximum amount of headers. On error returns ErrorSendingMessageInIBD
    pub fn ibd_send_get_block_headers_message(&self, last_hash: [u8; 32],  sync_node_index: usize) -> Result<(), NodeError> {

        let get_block_headers_msg = self.create_get_block_header_message(last_hash);
        //p
        println!("Mandamos {:?}", get_block_headers_msg);
        let mut stream = &self.tcp_streams[sync_node_index];
        
        match get_block_headers_msg.send_to(&mut stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInIBD),
        }
    }

    ///Creates a block downloader and returns it. On error returns NodeError
    fn create_block_downloader(&self, header_stream_index: usize) -> Result<(BlockDownloader, Arc<Mutex<Vec<Block>>>), NodeError>{
        let new_blocks :Vec<Block> = Vec::new();
        let safe_new_blocks = Arc::new(Mutex::from(new_blocks));
        let block_downloader = BlockDownloader::new(&self.tcp_streams, header_stream_index, &safe_new_blocks, self.logger.clone());
        match block_downloader{
            Ok(block_downloader) => Ok((block_downloader, safe_new_blocks)),
            Err(_) => Err(NodeError::ErrorCreatingBlockDownloader),
        }
    }
    
    ///Downloads the blocks from the node, starting from the given block hash. It ignores the messages that
    ///are not block messages, and only downloads blocks that are after the given time. On error returns NodeError
    fn download_headers_and_blocks(&mut self, block_downloader :&BlockDownloader, sync_node_index: usize) -> Result<(), NodeError> {
        let mut headers_received = self.block_headers.len();
        let mut last_hash = HASHEDGENESISBLOCK;
        if !self.block_headers.is_empty(){
            last_hash = self.block_headers[headers_received - 1].hash();
        }
        
        let mut request_block_hashes_bundle :Vec<[u8;32]> = Vec::new();
        let mut amount_of_blocks_to_download = 0;

        while headers_received == self.block_headers.len(){
            self.ibd_send_get_block_headers_message(last_hash, sync_node_index)?;

            let start_time = Instant::now();
            let target_duration = Duration::from_secs(HEADER_TIME_OUT);
            while self.receive_message(sync_node_index, true)? != "headers\0\0\0\0\0" {
                if Instant::now() - start_time > target_duration{
                    return Err(NodeError::ErrorReceivingHeadersMessageInIBD);
                }
            }
            
            let mut i = headers_received;
            headers_received += 2000;
            last_hash = self.block_headers[self.block_headers.len()-1].hash();

            if i == self.block_headers.len(){
                break;
            }

            if self.block_headers[i].time() > STARTING_BLOCK_TIME{
                while i < self.block_headers.len(){
                    if self.block_headers[i].time() > STARTING_BLOCK_TIME { 
                        amount_of_blocks_to_download += 1;
                        request_block_hashes_bundle.push(self.block_headers[i].hash());
                        if request_block_hashes_bundle.len() == MAX_BLOCK_BUNDLE{
                            if block_downloader.download_block_bundle(request_block_hashes_bundle).is_err(){
                                return Err(NodeError::ErrorDownloadingBlockBundle);
                            }
                            request_block_hashes_bundle = Vec::new();
                        }
                    }
                    i += 1;
                }
            }
            //p
            println!("#headers = {}", headers_received);
            self.logger.log(format!("Current amount of downloaded headers = {}", headers_received));
        }
        
        //p
        println!("# blocks  = {}", headers_received);
        self.logger.log(format!("Amount of blocks to download = {}", amount_of_blocks_to_download));
        
        if request_block_hashes_bundle.len() >0{
            if block_downloader.download_block_bundle(request_block_hashes_bundle).is_err(){
                return Err(NodeError::ErrorDownloadingBlockBundle);
            }
        }
        
        Ok(())
    }
    
    /// Writes the necessary headers and data into disk, to be able to continue the IBD from the last point. On error returns NodeError
    /// Both are written starting from the given positions.
    fn store_data_in_disk(&mut self, headers_starting_position: usize) -> Result<(), NodeError>{
        
        if self.data_handler.save_to_disk(&self.blockchain, &self.block_headers, headers_starting_position).is_err(){
            return Err(NodeError::ErrorSavingDataToDisk);
        }
        Ok(())
    }

    pub fn load_blocks_and_headers(&mut self)->Result<(), NodeError>{
        let headers = match self.data_handler.get_all_headers(){
            Ok(headers) => headers,
            Err(_) => return Err(NodeError::ErrorLoadingDataFromDisk),
        };

        let blocks = match self.data_handler.get_all_blocks(){
            Ok(blocks) => blocks,
            Err(_) => return Err(NodeError::ErrorLoadingDataFromDisk),
        };

        for block in blocks{
            _ = self.blockchain.insert(block.get_header().hash(), block);
        }
        self.block_headers.extend(headers);
        Ok(())
    }

    ///Asks the node for the block headers starting from the given block hash, and then downloads the blocks
    ///starting from the given time. On error returns NodeError
    pub fn initial_block_download(&mut self) -> Result<(), NodeError> {
        
        self.load_blocks_and_headers()?;
        let new_headers_starting_position = self.block_headers.len();

        let mut i = 1;
        let (mut block_downloader, mut safe_new_blocks) = self.create_block_downloader(i)?;
        
        while i < self.tcp_streams.len(){
            println!("\n{i}\n");
            match self.download_headers_and_blocks(&block_downloader, i){
                Ok(_) => break,
                Err(error) => { 
                    if let NodeError::ErrorDownloadingBlockBundle = error{
                        return Err(error);
                    }
                },
            };
            i += 1;
            if let Err(error) = block_downloader.finish_downloading(){
                self.logger.log_error(&error);
            }
            (block_downloader, safe_new_blocks) = self.create_block_downloader(i)?;
        }

        match block_downloader.finish_downloading(){
            Ok(_) => {
                let inner_vector = match safe_new_blocks.lock() {
                    Ok(inner) => inner,
                    Err(_) => return Err(NodeError::ErrorDownloadingBlockBundle),
                };
                for block in inner_vector.iter(){
                    let copied_block = match Block::from_bytes(&mut block.to_bytes()){
                        Ok(block) => block,
                        Err(_) => return Err(NodeError::ErrorDownloadingBlockBundle),
                    };
                    
                    if let Some(_) = self.blockchain.insert(copied_block.get_header().hash(), copied_block){
                        self.logger.log(String::from("Redownloaded a block that was already in the blockchain"));
                    }
                }
            },
            Err(_) => {return Err(NodeError::ErrorDownloadingBlockBundle)},
        }
        
        //p
        println!("# final de headers  = {}", self.block_headers.len());
        println!("# final de blocks  = {}", self.blockchain.len());
        self.logger.log(format!("Final amount of headers after IBD = {}", self.block_headers.len()));
        self.logger.log(format!("Final amount of blocks after IBD = {}", self.blockchain.len()));
        
        self.store_data_in_disk(new_headers_starting_position)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::sync::{Arc, Mutex};
    
    
    //test unitario de descargar un solo header


    #[test]
    fn ibd_test_1_can_download_blocks() -> Result<(), NodeError>{
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,3],
            local_port: 1003,
            log_path: String::from("src/node_log.txt"),
            begin_time: 1681084800,
        };
        let mut node = Node::new(config)?;
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, 0)?;
        while node.receive_message(0, true)? != "headers\0\0\0\0\0" {

        }
        
        assert!(node.block_headers.len() == 2000);
        Ok(())
    }

    #[test]
    fn ibd_test_2_can_download_blocks() -> Result<(), NodeError>{
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,2],
            local_port: 1002,
            log_path: String::from("src/node_log.txt"),
            begin_time: 1681084800,
        };
        let sync_node_index = 0;
        let mut node = Node::new(config)?;
        let vec = Vec::new();
        let safe_block_chain = Arc::new(Mutex::from(vec));
        let mut block_downloader = BlockDownloader::new(node.get_tcp_streams(), 0, &safe_block_chain, node.logger.clone()).unwrap();
        for _ in 0..1{
            node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, 0)?;
            while node.receive_message(sync_node_index, true)? != "headers\0\0\0\0\0" {

            }   
            for j in 0..125{
                let mut block_hashes_bundle :Vec<[u8;32]> = Vec::new();
                for i in 0..16{
                    block_hashes_bundle.push(node.block_headers[j*16 + i].hash());
                }
                block_downloader.download_block_bundle(block_hashes_bundle).unwrap();
            }

        }

        block_downloader.finish_downloading().unwrap();
        
        let blocks = safe_block_chain.lock().unwrap();
        println!("{}", blocks.len());
        
        assert!(blocks.len() == 2000);
        Ok(())
    }
}