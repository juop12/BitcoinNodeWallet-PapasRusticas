use crate::node::*;
use block_downloader::*;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const HASHEDGENESISBLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
    0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
];
const MAX_BLOCK_BUNDLE: usize = 16;
const MAXIMUM_PEER_TIME_OUT: u64 = 10;

impl Node {
    /// Creates a GetBlockHeadersMessage with the given hash
    fn create_get_block_header_message(&self, hash: [u8; 32]) -> GetBlockHeadersMessage {
        let block_header_hashes = vec![hash];
        let version = self.version as u32;
        let stopping_hash = [0_u8; 32];

        GetBlockHeadersMessage::new(version, block_header_hashes, stopping_hash)
    }

    /// Creates and sends a GetBlockHeadersMessage to the stream, always asking for the maximum amount of headers. On error returns ErrorSendingMessageInIBD
    pub fn ibd_send_get_block_headers_message(
        &self,
        last_hash: [u8; 32],
        sync_node_index: usize,
    ) -> Result<(), NodeError> {
        let get_block_headers_msg = self.create_get_block_header_message(last_hash);

        let mut stream = &self.tcp_streams[sync_node_index];

        match get_block_headers_msg.send_to(&mut stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInIBD),
        }
    }

    /// Creates a block downloader and returns it. On error returns NodeError
    fn create_block_downloader(
        &self,
        header_stream_index: usize,
    ) -> Result<(BlockDownloader, SafeVecBlock), NodeError> {
        let new_blocks: Vec<Block> = Vec::new();
        let safe_new_blocks = Arc::new(Mutex::from(new_blocks));
        let block_downloader = BlockDownloader::new(
            &self.tcp_streams,
            header_stream_index,
            &safe_new_blocks,
            &self.logger,
        );
        match block_downloader {
            Ok(block_downloader) => Ok((block_downloader, safe_new_blocks)),
            Err(_) => Err(NodeError::ErrorCreatingBlockDownloader),
        }
    }

    /// Requests block_downloader to download block bundles (16 blocks each),
    /// that were created after the starting_block_time.
    /// If at the end we do not have enough to form a full block bundle, then then unrequested block hashes are returned
    fn request_blocks(
        &mut self,
        mut i: usize,
        mut request_block_hashes_bundle: Vec<[u8; 32]>,
        block_downloader: &BlockDownloader,
        total_amount_of_blocks: &mut usize,
    ) -> Result<Vec<[u8; 32]>, NodeError> {
        while i < self.block_headers.len() {
            if self.block_headers[i].time > self.starting_block_time {
                *total_amount_of_blocks += 1;
                request_block_hashes_bundle.push(self.block_headers[i].hash());
                if request_block_hashes_bundle.len() == MAX_BLOCK_BUNDLE {
                    if block_downloader
                        .download_block_bundle(request_block_hashes_bundle)
                        .is_err()
                    {
                        return Err(NodeError::ErrorDownloadingBlockBundle);
                    }
                    request_block_hashes_bundle = Vec::new();
                }
            }
            i += 1;
        }

        Ok(request_block_hashes_bundle)
    }

    /// Receives messages from a given peer till it receives a headersMessage or 30 seconds have passed
    fn receive_headers_message(&mut self, sync_node_index: usize, peer_timeout: u64) -> Result<(), NodeError> {
        let mut start_time = Instant::now();
        let target_duration = Duration::from_secs(peer_timeout);
        while self.receive_message(sync_node_index, true)? != "headers\0\0\0\0\0" {
            if Instant::now() - start_time > target_duration {
                self.logger.log(format!("Peer {} timed_out switching peers", sync_node_index));
                return Err(NodeError::ErrorReceivingHeadersMessageInIBD);
            }
            start_time = Instant::now();
        }
        if Instant::now() - start_time > target_duration {
            self.logger.log(format!("Peer {} timed_out switching peers", sync_node_index));
            return Err(NodeError::ErrorReceivingHeadersMessageInIBD);
        }
        Ok(())
    }

    /// Downloads the blocks from the node, starting from the given block hash. It ignores the messages that
    /// are not block messages, and only downloads blocks that are after the given time. On error returns NodeError
    fn download_headers_and_blocks(
        &mut self,
        block_downloader: &BlockDownloader,
        sync_node_index: usize,
        peer_timeout: u64
    ) -> Result<(), NodeError> {
        let mut headers_received = self.block_headers.len();
        let mut last_hash = HASHEDGENESISBLOCK;
        if !self.block_headers.is_empty() {
            last_hash = self.block_headers[headers_received - 1].hash();
        }

        let mut request_block_hashes_bundle: Vec<[u8; 32]> = Vec::new();
        let mut total_amount_of_blocks = self.blockchain.len();

        while headers_received == self.block_headers.len() {
            self.ibd_send_get_block_headers_message(last_hash, sync_node_index)?;

            self.receive_headers_message(sync_node_index, peer_timeout)?;

            let i = headers_received;
            headers_received += 2000;
            last_hash = self.block_headers[self.block_headers.len() - 1].hash();

            if i == self.block_headers.len() {
                break;
            }

            request_block_hashes_bundle = self.request_blocks(
                i,
                request_block_hashes_bundle,
                block_downloader,
                &mut total_amount_of_blocks,
            )?;
            //p
            println!("#headers = {}", headers_received);
            self.logger.log(format!(
                "Current ammount of downloaded headers = {}",
                headers_received
            ));
        }

        //p
        println!("#blocks = {}", total_amount_of_blocks);
        self.logger.log(format!(
            "Total ammount of blocks = {}",
            total_amount_of_blocks
        ));

        if !request_block_hashes_bundle.is_empty()
            && block_downloader
                .download_block_bundle(request_block_hashes_bundle)
                .is_err()
        {
            return Err(NodeError::ErrorDownloadingBlockBundle);
        }

        Ok(())
    }

    /// Writes the necessary headers into disk, to be able to continue the IBD from the last point. 
    /// On error returns NodeError. Written starting from the given positions.
    fn store_headers_in_disk(&mut self) -> Result<(), NodeError> {
        self.data_handler.save_headers_to_disk( &self.block_headers,
self.headers_in_disk).map_err(|_| NodeError::ErrorSavingDataToDisk)
    }

    /// Writes the necessary blocks into disk, to be able to continue the IBD from the last point. 
    /// On error returns NodeError. Written starting from the given positions.
    fn store_blocks_in_disk(&mut self) -> Result<(), NodeError> {
        self.data_handler.save_blocks_to_disk(&self.blockchain, &self.block_headers,self.headers_in_disk).map_err(|_| NodeError::ErrorSavingDataToDisk)

    }

    /// Loads the blocks and headers from disk. On error returns NodeError
    pub fn load_blocks_and_headers(&mut self) -> Result<(), NodeError> {
        let headers = match self.data_handler.get_all_headers() {
            Ok(headers) => headers,
            Err(_) => return Err(NodeError::ErrorLoadingDataFromDisk),
        };

        let blocks = match self.data_handler.get_all_blocks() {
            Ok(blocks) => blocks,
            Err(_) => return Err(NodeError::ErrorLoadingDataFromDisk),
        };

        for block in blocks {
            _ = self.blockchain.insert(block.get_header().hash(), block);
        }
        self.block_headers.extend(headers);
        Ok(())
    }

    /// Downloads block and headers from a given peer.If a problem occurs while downloading headers it continues asking to another peer.
    fn start_downloading(&mut self) -> Result<(BlockDownloader, SafeVecBlock), NodeError> {
        let mut i = 0;
        let (mut block_downloader, mut safe_new_blocks) = self.create_block_downloader(i)?;

        let mut peer_time_out = 1;
        while peer_time_out < MAXIMUM_PEER_TIME_OUT {
            println!("\n{i}\n");
            match self.download_headers_and_blocks(&block_downloader, i, peer_time_out) {
                Ok(_) => break,
                Err(error) => {
                    if let NodeError::ErrorDownloadingBlockBundle = error {
                        return Err(error);
                    }
                }
            };
            i += 1;
            if i >= self.tcp_streams.len(){
                i = 0;
                peer_time_out +=1;
                self.logger.log(format!("Reducing time standards, new peer_time_out = {} seconds", peer_time_out));
            }
            if let Err(error) = block_downloader.finish_downloading() {
                self.logger.log_error(&error);
            }
            (block_downloader, safe_new_blocks) = self.create_block_downloader(i)?;
        }
        Ok((block_downloader, safe_new_blocks))
    }

    /// Tells a block downloader that no more bundles will be sent, and after it finishes it inserts downloaded blocks into the blockchain.
    fn ibd_finish_downloading(
        &mut self,
        mut block_downloader: BlockDownloader,
        safe_new_blocks: Arc<Mutex<Vec<Block>>>,
    ) -> Result<(), NodeError> {
        match block_downloader.finish_downloading() {
            Ok(_) => {
                let inner_vector = match safe_new_blocks.lock() {
                    Ok(inner) => inner,
                    Err(_) => return Err(NodeError::ErrorDownloadingBlockBundle),
                };
                for block in inner_vector.iter() {
                    let copied_block = match Block::from_bytes(&block.to_bytes()) {
                        Ok(block) => block,
                        Err(_) => return Err(NodeError::ErrorDownloadingBlockBundle),
                    };

                    if self
                        .blockchain
                        .insert(copied_block.get_header().hash(), copied_block)
                        .is_some()
                    {
                        self.logger.log(String::from(
                            "Redownloaded a block that was already in the blockchain",
                        ));
                    }
                }
            }
            Err(_) => return Err(NodeError::ErrorDownloadingBlockBundle),
        }
        Ok(())
    }

    /// Asks the node for the block headers starting from the given block hash,
    /// and then downloads the blocks starting from the given time.
    /// On error returns NodeError
    pub fn initial_block_download(&mut self) -> Result<(), NodeError> {
        self.logger.log(String::from("Started loading data from disk"));
        self.load_blocks_and_headers()?;
        self.logger.log(String::from("Finished loading data from disk"));
        self.headers_in_disk = self.block_headers.len();
        
        let (block_downloader, safe_new_blocks) = self.start_downloading()?;
        
        self.logger.log(String::from("Started storing headers to disk"));
        self.store_headers_in_disk()?;
        self.logger.log(String::from("Finished storing headers to disk"));
        
        self.ibd_finish_downloading(block_downloader, safe_new_blocks)?;
        
        //p
        println!("# final de headers  = {}", self.block_headers.len());
        println!("# final de blocks  = {}", self.blockchain.len());
        self.logger.log(format!(
            "Final amount of headers after IBD = {}",
            self.block_headers.len()
        ));
        self.logger.log(format!(
            "Final amount of blocks after IBD = {}",
            self.blockchain.len()
        ));
        
        self.logger.log(String::from("Started storing blocks to disk"));
        self.store_blocks_in_disk()?;
        self.logger.log(String::from("Finished storing blocks to disk"));
        
        self.headers_in_disk = self.block_headers.len();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    const STARTING_BLOCK_TIME: u32 = 1681084800;
    const HEADERS_FILE_PATH: &str = "tests_txt/ibd_test_headers.bin";
    const BLOCKS_FILE_PATH: &str = "tests_txt/ibd_test_blocks.bin";

    #[test]
    fn ibd_test_1_can_download_headers() -> Result<(), NodeError> {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127, 0, 0, 3],
            local_port: 1003,
            log_path: String::from("tests_txt/ibd_test_1_log.txt"),
            begin_time: STARTING_BLOCK_TIME,
            headers_path: String::from(HEADERS_FILE_PATH),
            blocks_path: String::from(BLOCKS_FILE_PATH),
            ipv6_enabled: false,
        };
        let mut node = Node::new(config)?;
        let mut i = 0;
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, i)?;
        while let Err(_) = node.receive_headers_message(i, 15) {
            i += 1;
            node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, i)?;
        }

        assert!(node.block_headers.len() == 2000);
        Ok(())
    }

    #[test]
    fn ibd_test_2_can_download_2000_blocks() -> Result<(), NodeError> {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127, 0, 0, 2],
            local_port: 1002,
            log_path: String::from("tests_txt/ibd_test_2_log.txt"),
            begin_time: STARTING_BLOCK_TIME,
            headers_path: String::from(HEADERS_FILE_PATH),
            blocks_path: String::from(BLOCKS_FILE_PATH),
            ipv6_enabled: false,
        };
        
        let mut node = Node::new(config)?;
        let vec = Vec::new();
        let safe_block_chain = Arc::new(Mutex::from(vec));
        let mut block_downloader = BlockDownloader::new(
            node.get_tcp_streams(),
            0,
            &safe_block_chain,
            &node.logger.clone(),
        )
        .unwrap();

        let mut sync_node_index = 0;
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        while let Err(_) = node.receive_headers_message(sync_node_index,15) {
            sync_node_index += 1;
            node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        }

        for j in 0..125 {
            let mut block_hashes_bundle: Vec<[u8; 32]> = Vec::new();
            for i in 0..16 {
                block_hashes_bundle.push(node.block_headers[j * 16 + i].hash());
            }
            block_downloader
                .download_block_bundle(block_hashes_bundle)
                .unwrap();
        }

        block_downloader.finish_downloading().unwrap();

        let blocks = safe_block_chain.lock().unwrap();
        println!("{}", blocks.len());

        assert!(blocks.len() == 2000);
        Ok(())
    }

    #[test]
    fn ibd_test_3_can_download_2000_blocks() -> Result<(), NodeError> {
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127, 0, 0, 2],
            local_port: 1002,
            log_path: String::from("tests_txt/ibd_test_2_log.txt"),
            begin_time: STARTING_BLOCK_TIME,
            headers_path: String::from(HEADERS_FILE_PATH),
            blocks_path: String::from(BLOCKS_FILE_PATH),
            ipv6_enabled: false,
        };
        let mut node = Node::new(config)?;

        let mut sync_node_index = 0;
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        while let Err(_) = node.receive_headers_message(sync_node_index,15) {
            sync_node_index += 1;
            node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        }

        let mut block_hashes_bundle: Vec<[u8; 32]> = Vec::new();
        for i in 0..16 {
            block_hashes_bundle.push(node.block_headers[16 + i].hash());
        }

        get_blocks_from_bundle(block_hashes_bundle, &mut node.tcp_streams[sync_node_index], &node.logger);

        Ok(())
    }
}
