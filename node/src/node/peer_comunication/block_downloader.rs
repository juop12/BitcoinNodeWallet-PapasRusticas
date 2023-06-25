use crate::{
    messages::{get_data_message::*, message_trait::Message},
    node::{*, handshake::PEER_TIMEOUT},
    utils::btc_errors::BlockDownloaderError,
};

use std::{
    net::TcpStream,
    sync::{mpsc, Arc, Mutex}, time::{Instant, Duration},
};
use workers::*;

/// Gets a bundle from the shared reference and logs the errors
fn get_bundle(id: usize, receiver: &SafeReceiver, logger: &Logger) -> Option<Bundle> {
    let bundle = match receiver.lock() {
        Ok(rec_lock) => match rec_lock.recv() {
            Ok(bundle) => bundle,
            Err(error) => {
                logger.log(format!("Worker {id} failed: {:?}", error));
                return None;
            }
        },
        Err(error) => {
            logger.log(format!("Worker {id} failed: {:?}", error));
            return None;
        }
    };
    Some(bundle)
}

/// Main loop that worker's thread executes. It gets a bundle from the shared channel,
/// gets the blocks from it's peer, and saves them to the shared reference block vector.
/// If anything fails along the way it loggs acordingly, as well as other things like
/// received messages.
/// It returs an option representing wheter the loop must stop (Some) or continue None.
/// The bool stored in Some represents whether the stop is a "Gracefull stop", meaning
/// the thread must return true (for example when receiving an end of channel), or an
/// "Ungracefull stop" meaning the loop failed at some point
pub fn block_downloader_thread_loop(
    id: usize,
    receiver: &SafeReceiver,
    stream: &mut TcpStream,
    safe_headers: &SafeVecHeader,
    safe_block_chain: &SafeBlockChain,
    missed_bundles_sender: &mpsc::Sender<Bundle>,
    logger: &Logger,
) -> Stops {
    let bundle = match get_bundle(id, receiver, logger) {
        Some(bundle) => bundle,
        None => return Stops::UngracefullStop,
    };

    //si se recibe un end_of_channel
    if bundle.is_empty() {
        return Stops::GracefullStop;
    }

    let aux_bundle = bundle.clone();

    match get_blocks_from_bundle(bundle, stream, &safe_headers, &safe_block_chain,logger) {
        Ok(blocks) => blocks,
        Err(error) => {
            if let Err(error) = missed_bundles_sender.send(aux_bundle) {
                logger.log(format!(
                    "Worker {id} failed sending missed bundle: {:?}",
                    error
                ));
            }

            if let BlockDownloaderError::BundleNotFound = error {
                logger.log(format!("Worker {id} did not find bundle"));
                return Stops::Continue;
            } else {
                logger.log(format!("Worker {id} failed: {:?}", error));
                return Stops::UngracefullStop;
            }
        }
    };
    
    Stops::Continue
}

//=====================================================================================

#[derive(Debug)]
pub struct BlockDownloader {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Bundle>,
    safe_headers: SafeVecHeader,
    safe_blockchain: SafeBlockChain,
    missed_bundles_receiver: mpsc::Receiver<Bundle>,
    logger: Logger,
}

impl BlockDownloader {
    /// Creates a new thread pool with the specified size, it must be greater than zero.
    pub fn new(
        outbound_connections: &Vec<TcpStream>,
        header_stream_index: usize,
        safe_headers: &SafeVecHeader,
        safe_blockchain: &SafeBlockChain,
        logger: &Logger,
    ) -> Result<BlockDownloader, BlockDownloaderError> {
        let connections_ammount = outbound_connections.len();
        if connections_ammount == 0 {
            return Err(BlockDownloaderError::ErrorInvalidCreationSize);
        }

        let (sender, receiver) = mpsc::channel();
        let (missed_bundles_sender, missed_bundles_receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(connections_ammount);

        //No tomamos el tcp stream que se esta usando para descargar headers, porque se usa para descargar headers.
        for (id, stream) in outbound_connections
            .iter()
            .enumerate()
            .take(connections_ammount)
        {
            if id == header_stream_index {
                continue;
            }

            let current_stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => {
                    logger.log_error(&BlockDownloaderError::ErrorCreatingWorker);
                    continue;
                }
            };

            let worker = Worker::new_block_downloader_worker(
                id,
                receiver.clone(),
                current_stream,
                safe_headers.clone(),
                safe_blockchain.clone(),
                missed_bundles_sender.clone(),
                logger.clone(),
            );

            workers.push(worker);
        }

        Ok(BlockDownloader {
            workers,
            sender,
            safe_headers: safe_headers.clone(),
            safe_blockchain: safe_blockchain.clone(),
            missed_bundles_receiver,
            logger: logger.clone(),
        })
    }

    /// Receives a function or closure that receives no parameters and executes them in a diferent thread using workers.x
    pub fn download_block_bundle(&self, bundle: Vec<[u8; 32]>) -> Result<(), BlockDownloaderError> {
        if bundle.is_empty() {
            return Ok(());
        }

        match self.sender.send(bundle) {
            Ok(_) => Ok(()),
            Err(_err) => Err(BlockDownloaderError::ErrorSendingToThread),
        }
    }

    /// Writes an empty vector to the channel of the workers, so they can finish their execution. It works as
    /// a way to stop the threads execution. On error, it returns BlockDownloaderError.
    pub fn finish_downloading(&mut self) -> Result<(), BlockDownloaderError> {
        let cantidad_workers = self.workers.len();
        let mut working_peer_conection = None;
        for _ in 0..cantidad_workers {
            let end_of_channel: Vec<[u8; 32]> = Vec::new();
            if self.sender.send(end_of_channel).is_err() {
                self.logger
                    .log(String::from("Falló en el envio al end of channel"));
                return Err(BlockDownloaderError::ErrorSendingToThread);
            }

            while let Ok(bundle) = self.missed_bundles_receiver.try_recv() {
                self.download_block_bundle(bundle)?;
            }

            let mut joined_a_worker = false;

            while !joined_a_worker {
                let worker = self.workers.remove(0);
                if worker.is_finished() {
                    let stream_op = worker.join_thread()?;
                    if working_peer_conection.is_none() {
                        working_peer_conection = stream_op;
                    };
                    joined_a_worker = true;
                } else {
                    self.workers.push(worker);
                }
            }
        }

        let mut stream = match working_peer_conection {
            Some(stream) => stream,
            None => return Err(BlockDownloaderError::ErrorAllWorkersFailed),
        };

        while let Ok(bundle) = self.missed_bundles_receiver.try_recv() {
            get_blocks_from_bundle(bundle, &mut stream, &self.safe_headers, &self.safe_blockchain,&self.logger)?;
        }

        Ok(())
    }
}

fn receive_block(
    stream: &mut TcpStream,
    safe_headers: &SafeVecHeader,
    safe_blockchain: &SafeBlockChain,
    logger: &Logger,
) -> Result<(), BlockDownloaderError> {
    
    let dummy_pending_tx = Arc::new(Mutex::new(HashMap::new()));
    
    let start_time = Instant::now(); 
    while start_time.elapsed() < PEER_TIMEOUT{
        match receive_message(stream, safe_headers, safe_blockchain, &dummy_pending_tx, logger, true){
            Ok(message_cmd) => {
                match message_cmd.as_str() {
                    "block\0\0\0\0\0\0\0" => return Ok(()),
                    "notfound\0\0\0\0" => return Err(BlockDownloaderError::BundleNotFound),
                    _ => {},
                };
            }
            Err(error) => {
                match error{
                    NodeError::ErrorDownloadingBlockBundle => return Err(BlockDownloaderError::BundleNotFound),
                    NodeError::ErrorValidatingBlock => return Err(BlockDownloaderError::ErrorValidatingBlock),
                    _ => return Err(BlockDownloaderError::ErrorReceivingBlockMessage),
                }
            }
        }
    }
    Err(BlockDownloaderError::ErrorReceivingBlockMessage)
}


/// Sends a getdata message to the stream, requesting the blocks with the specified hashes.
/// Returns an error if it was not possible to send the message.
pub fn send_get_data_message_for_blocks(
    hashes: Vec<[u8; 32]>,
    stream: &mut TcpStream,
) -> Result<(), BlockDownloaderError> {
    let get_data_message = GetDataMessage::create_message_inventory_block_type(hashes);

    match get_data_message.send_to(stream) {
        Ok(_) => Ok(()),
        Err(_) => Err(BlockDownloaderError::ErrorSendingMessageBlockDownloader),
    }
}

/// Receives a vector of block hashes and a TcpStream, and returns a vector of blocks that were requested to the stream
fn get_blocks_from_bundle(
    requested_block_hashes: Vec<[u8; 32]>,
    stream: &mut TcpStream,
    safe_headers: &SafeVecHeader,
    safe_blockchain: &SafeBlockChain,
    logger: &Logger,
) -> Result<(), BlockDownloaderError> {
    if requested_block_hashes.is_empty(){
        return Ok(());
    }
    let amount_of_hashes = requested_block_hashes.len();
    send_get_data_message_for_blocks(requested_block_hashes, stream)?;
    for _ in 0..amount_of_hashes {
        receive_block(stream, safe_headers, safe_blockchain,logger)?;
    }

    Ok(())
}
