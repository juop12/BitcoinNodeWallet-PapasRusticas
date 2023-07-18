use crate::{node::*, utils::{log, LoadingScreenInfo}};
use block_downloader::*;
use std::{time::{Duration, Instant}, thread::JoinHandle};
use std::thread;
use glib::{Sender, Receiver};
use crate::utils::ui_communication_protocol::UIResponse;

const HASHEDGENESISBLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
    0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
];
const MAX_BLOCK_BUNDLE: usize = 16;
const MAXIMUM_PEER_TIME_OUT: u64 = 10;
const REFRESH_BLOCK_DOWNLOAD_PROGRESS_FOR_UI: Duration = Duration::from_secs(1);

fn send_ibd_information_to_ui(sender_to_ui: GlibSender<UIResponse>, blockchain: Arc<Mutex<HashMap<[u8;32], Block>>>, total_blocks: usize, starting_block_count: usize) -> Result<JoinHandle<()>, NodeError>{
    let message_to_ui = LoadingScreenInfo::StartedBlockDownload(total_blocks);
    let started_downloading = "Started downloading blocks";
    sender_to_ui.send(UIResponse::LoadingScreenUpdate(message_to_ui)).expect("Error sending to UI thread");
    sender_to_ui.send(UIResponse::LoadingScreenUpdate(LoadingScreenInfo::UpdateLabel(started_downloading.to_string()))).expect("Error sending to UI thread");
    let sender_clone = sender_to_ui.clone();

    let join_handle = thread::spawn(move || { loop {
        thread::sleep(REFRESH_BLOCK_DOWNLOAD_PROGRESS_FOR_UI);
        match blockchain.lock() {
            Ok(blockchain) => {
                let current_block_count = blockchain.len() - starting_block_count;
                let message_to_ui = LoadingScreenInfo::DownloadedBlocks(current_block_count);
                sender_clone.send(UIResponse::LoadingScreenUpdate(message_to_ui)).expect("Error sending to UI thread");
                if current_block_count == total_blocks {
                    sender_to_ui.send(UIResponse::LoadingScreenUpdate(LoadingScreenInfo::FinishedBlockDownload)).expect("Error sending to UI thread");
                    break;
                }
            },
            Err(_) => {},
        }   
    }
    });
    Ok(join_handle)
}

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

        let mut stream = &self.initial_peers[sync_node_index];

        match get_block_headers_msg.send_to(&mut stream) {
            Ok(_) => Ok(()),
            Err(_) => Err(NodeError::ErrorSendingMessageInIBD),
        }
    }

    /// Creates a block downloader and returns it. On error returns NodeError
    fn create_block_downloader(
        &self,
        header_stream_index: usize,
    ) -> Result<BlockDownloader, NodeError> {
        let block_downloader = BlockDownloader::new(
            &self.initial_peers,
            header_stream_index,
            &self.block_headers,
            &self.blockchain,
            &self.logger,
        );
        match block_downloader {
            Ok(block_downloader) => Ok(block_downloader),
            Err(_) => Err(NodeError::ErrorCreatingBlockDownloader),
        }
    }

    /// Receives messages from a given peer till it receives a headersMessage or 30 seconds have passed
    fn receive_headers_message(
        &mut self,
        sync_node_index: usize,
        peer_timeout: u64,
    ) -> Result<(), NodeError> {
        let mut start_time = Instant::now();
        let target_duration = Duration::from_secs(peer_timeout);
        while self.receive_message(sync_node_index, true)? != "headers\0\0\0\0\0" {
            if Instant::now() - start_time > target_duration {
                self.logger.log(format!(
                    "Peer {} timed_out switching peers",
                    sync_node_index
                ));
                return Err(NodeError::ErrorReceivingHeadersMessageInIBD);
            }
            start_time = Instant::now();
        }
        if Instant::now() - start_time > target_duration {
            self.logger.log(format!(
                "Peer {} timed_out switching peers",
                sync_node_index
            ));
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
        peer_timeout: u64,
        first_downloaded_block_index: &mut i32,
        starting_block_count: usize,
    ) -> Result<JoinHandle<()>, NodeError> {
        let mut headers_received = self.get_block_headers()?.len();
        let mut last_hash = HASHEDGENESISBLOCK;
        if !self.get_block_headers()?.is_empty() {
            last_hash = self.get_block_headers()?[headers_received - 1].hash();
        }
        let mut request_block_hashes_bundle: Vec<[u8; 32]> = Vec::new();
        let mut total_amount_of_blocks = self.get_block_headers()?.len();
        while headers_received == self.get_block_headers()?.len() {
            self.ibd_send_get_block_headers_message(last_hash, sync_node_index)?;

            self.receive_headers_message(sync_node_index, peer_timeout)?;

            let i = headers_received;
            headers_received += 2000;
            let block_headers = self.get_block_headers()?;
            last_hash = block_headers[block_headers.len() - 1].hash();

            if i == block_headers.len() {
                break;
            }

            request_block_hashes_bundle = request_blocks(
                i,
                &block_headers,
                request_block_hashes_bundle,
                block_downloader,
                &mut total_amount_of_blocks,
                self.starting_block_time,
                first_downloaded_block_index
            )?;
            let log_str = format!(
                "Current amount of downloaded headers = {}",
                headers_received
            );
            self.log_and_send_to_ui(&log_str, &log_str);
        }
        let total_blocks = self.get_block_headers()?.len();
        let mut amount_of_blocks_to_download = 0;
        if *first_downloaded_block_index != -1 {
            amount_of_blocks_to_download = total_blocks - ((*first_downloaded_block_index) as usize); // the block in the position 0 is the genesis block so we dont add 1
        }
        self.logger.log(format!(
            "Total amount of blocks to download = {}",
            amount_of_blocks_to_download
        ));
        let thread_join = send_ibd_information_to_ui(self.sender_to_ui.clone(), self.blockchain.clone(), amount_of_blocks_to_download, starting_block_count)?;

        if !request_block_hashes_bundle.is_empty()
            && block_downloader
                .download_block_bundle(request_block_hashes_bundle)
                .is_err()
        {
            return Err(NodeError::ErrorDownloadingBlockBundle);
        }
        // if thread_join.join().is_err() {
        //     self.logger.log(format!(
        //         "Error joining thread that sends IBD information to UI",
        //     ));
        // }
        Ok(thread_join)
    }

    /// Writes the necessary headers into disk, to be able to continue the IBD from the last point.
    /// On error returns NodeError. Written starting from the given positions.
    pub fn store_headers_in_disk(&mut self) -> Result<(), NodeError> {
        self.data_handler
            .save_headers_to_disk(&self.block_headers, self.headers_in_disk)
            .map_err(|_| NodeError::ErrorSavingDataToDisk)
    }

    /// Writes the necessary blocks into disk, to be able to continue the IBD from the last point.
    /// On error returns NodeError. Written starting from the given positions.
    pub fn store_blocks_in_disk(&mut self) -> Result<(), NodeError> {
        self.data_handler
            .save_blocks_to_disk(&self.blockchain, &self.block_headers, self.headers_in_disk)
            .map_err(|_| NodeError::ErrorSavingDataToDisk)
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
            _ = self
                .get_blockchain()?
                .insert(block.get_header().hash(), block);
        }
        self.get_block_headers()?.extend(headers);
        Ok(())
    }

    /// Downloads block and headers from a given peer.If a problem occurs while downloading headers it continues asking to another peer.
    fn start_downloading(&mut self, starting_block_count: usize) -> Result<(BlockDownloader, Option<JoinHandle<()>>), NodeError> {
        let mut i = 0;
        let mut block_downloader = self.create_block_downloader(i)?;
        let mut first_downloaded_block_index: i32 = -1;
        let mut peer_time_out = 1;
        let mut thread_join: Option<JoinHandle<()>> = None;
        while peer_time_out < MAXIMUM_PEER_TIME_OUT {
            println!("\n{i}\n");
            match self.download_headers_and_blocks(&block_downloader, i, peer_time_out, &mut first_downloaded_block_index, starting_block_count) {
                Ok(join) => {thread_join = Some(join);
                    break;
                }
                Err(error) => {
                    if let NodeError::ErrorDownloadingBlockBundle = error {
                        return Err(error);
                    }
                }
            };
            i += 1;
            if i >= self.initial_peers.len() {
                i = 0;
                peer_time_out += 1;
                self.logger.log(format!(
                    "Reducing time standards, new peer_time_out = {} seconds",
                    peer_time_out
                ));
            }
            if let Err(error) = block_downloader.finish_downloading() {
                self.logger.log_error(&error);
            }
            block_downloader = self.create_block_downloader(i)?;
        }
        Ok((block_downloader, thread_join))
    }

    /// Asks the node for the block headers starting from the given block hash,
    /// and then downloads the blocks starting from the given time.
    /// On error returns NodeError
    pub fn initial_block_download(&mut self) -> Result<(), NodeError> {
        let mut log_str = "Started loading data from disk";
        self.log_and_send_to_ui(log_str, log_str);

        self.load_blocks_and_headers()?;
        log_str = "Finished loading data from disk";
        self.log_and_send_to_ui(log_str, log_str);

        let mut aux_len = self.get_block_headers()?.len();
        self.headers_in_disk = aux_len;
        let starting_block_count = self.get_blockchain()?.len();

        let (mut block_downloader, thread_join) = self.start_downloading(starting_block_count)?;

        block_downloader
            .finish_downloading()
            .map_err(|_| NodeError::ErrorDownloadingBlockBundle)?;

        if let Some(join) = thread_join {
            join.join().map_err(|_| NodeError::ErrorJoiningThread)?;
        }

        log_str = "Started storing headers to disk";
        self.log_and_send_to_ui(log_str, log_str);
        
        self.store_headers_in_disk()?;
        log_str = "Finished storing headers to disk";
        self.log_and_send_to_ui(log_str, log_str);

        self.logger.log(format!(
            "Final amount of headers after IBD = {}",
            self.get_block_headers()?.len()
        ));
        self.logger.log(format!(
            "Final amount of blocks after IBD = {}",
            self.get_blockchain()?.len()
        ));
        self.sender_to_ui.send(UIResponse::LoadingScreenUpdate(LoadingScreenInfo::FinishedBlockDownload)).map_err(|_| NodeError::ErrorSendingThroughChannel)?;

        log_str = "Started storing blocks to disk";
        self.log_and_send_to_ui(log_str, log_str);

        self.store_blocks_in_disk()?;

        log_str = "Finished storing blocks to disk";
        self.log_and_send_to_ui(log_str, log_str);


        aux_len = self.get_block_headers()?.len();
        self.headers_in_disk = aux_len;

        self.last_proccesed_block = aux_len;
        Ok(())
    }
}

/// Requests block_downloader to download block bundles (16 blocks each),
/// that were created after the starting_block_time.
/// If at the end we do not have enough to form a full block bundle, then then unrequested block hashes are returned
fn request_blocks(
    mut i: usize,
    block_headers: &Vec<BlockHeader>,
    mut request_block_hashes_bundle: Vec<[u8; 32]>,
    block_downloader: &BlockDownloader,
    total_amount_of_blocks: &mut usize,
    starting_block_time: u32,
    first_downloaded_block_index: &mut i32
) -> Result<Vec<[u8; 32]>, NodeError> {
    while i < block_headers.len() {
        if block_headers[i].time > starting_block_time {
            if *first_downloaded_block_index == -1 {
                *first_downloaded_block_index = i as i32;
            }
            *total_amount_of_blocks += 1;
            request_block_hashes_bundle.push(block_headers[i].hash());
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let mut node = Node::new(config, sx)?;
        let mut i = 0;
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, i)?;
        while let Err(_) = node.receive_headers_message(i, 15) {
            i += 1;
            node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, i)?;
        }

        assert!(node.get_block_headers()?.len() == 2000);
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
        let (sx, _rx) = glib::MainContext::channel::<UIResponse>(glib::PRIORITY_DEFAULT);
        let mut node = Node::new(config, sx)?;
        let mut block_downloader = BlockDownloader::new(
            &node.initial_peers,
            0,
            &node.block_headers,
            &node.blockchain,
            &node.logger.clone(),
        )
        .unwrap();

        let mut sync_node_index = 0;
        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        while let Err(_) = node.receive_headers_message(sync_node_index, 15) {
            sync_node_index += 1;
            node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK, sync_node_index)?;
        }

        for j in 0..125 {
            let mut block_hashes_bundle: Vec<[u8; 32]> = Vec::new();
            for i in 0..16 {
                block_hashes_bundle.push(node.get_block_headers()?[j * 16 + i].hash());
            }
            block_downloader
                .download_block_bundle(block_hashes_bundle)
                .unwrap();
        }

        block_downloader.finish_downloading().unwrap();

        let blocks = node.get_blockchain()?;
        println!("{}", blocks.len());

        assert!(blocks.len() == 2000);
        Ok(())
    }
}
