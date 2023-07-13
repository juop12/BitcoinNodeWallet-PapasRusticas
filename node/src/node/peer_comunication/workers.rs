use crate::{node::*, utils::btc_errors::BlockDownloaderError};

use std::{
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use block_downloader::block_downloader_thread_loop;
use peer_comunicator::message_receiver_thread_loop;
pub type FinishedIndicator = Arc<Mutex<bool>>;

pub enum Stops {
    GracefullStop,
    UngracefullStop,
    Continue,
}

impl Stops {
    fn log_message(&self, id: usize) -> String {
        match *self {
            Stops::GracefullStop => format!("Worker {} finished gracefully", id),
            Stops::UngracefullStop => format!("Worker {} finished ungracefully", id),
            Stops::Continue => String::new(),
        }
    }
}

/// Struct that represents a worker thread in the thread pool.
#[derive(Debug)]
pub struct Worker {
    thread: thread::JoinHandle<Option<TcpStream>>,
    _id: usize,
}

pub type Bundle = Vec<[u8; 32]>;
pub type SafeReceiver = Arc<Mutex<mpsc::Receiver<Bundle>>>;

impl Worker {
    ///Creates a worker which attempts to execute tasks received trough the channel in a loop
    pub fn new_block_downloader_worker(
        id: usize,
        receiver: SafeReceiver,
        mut stream: TcpStream,
        safe_headers: SafeVecHeader,
        safe_blockchain: SafeBlockChain,
        missed_bundles_sender: mpsc::Sender<Bundle>,
        logger: Logger,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let stop = block_downloader_thread_loop(
                id,
                &receiver,
                &mut stream,
                &safe_headers,
                &safe_blockchain,
                &missed_bundles_sender,
                &logger,
            );

            match stop {
                Stops::GracefullStop => {
                    logger.log(Stops::GracefullStop.log_message(id));
                    return Some(stream);
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(id));
                    return None;
                }
                Stops::Continue => continue,
            }
        });

        Worker { _id: id, thread }
    }

    /// Creates a worker for a MessageReceiver
    pub fn new_message_receiver_worker(
        mut stream: TcpStream,
        safe_block_headers: SafeVecHeader,
        safe_block_chain: SafeBlockChain,
        safe_pending_tx: SafePendingTx,
        logger: Logger,
        finished: FinishedIndicator,
        id: usize,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            logger.log(format!("Sigo vivo: {}", id));
            match message_receiver_thread_loop(
                &mut stream,
                &safe_block_headers,
                &safe_block_chain,
                &safe_pending_tx,
                &logger,
                &finished,
            ) {
                Stops::GracefullStop => {
                    logger.log(Stops::GracefullStop.log_message(id));
                    return Some(stream);
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(id));
                    return None;
                }
                Stops::Continue => continue,
            }
        });

        Worker { thread, _id: id }
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    pub fn join_thread(self) -> Result<Option<TcpStream>, BlockDownloaderError> {
        match self.thread.join() {
            Ok(stream) => Ok(stream),
            Err(_) => Err(BlockDownloaderError::ErrorWrokerPaniced),
        }
    }

    /// Returns true if the thread has finished its execution.
    pub fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }
}
