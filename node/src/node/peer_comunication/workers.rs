use crate::{node::*, utils::{btc_errors::BlockDownloaderError, log, PeerComunicatorError, WorkerError}};

use std::{
    net::TcpStream,
    sync::{mpsc::{self, RecvTimeoutError}, Arc, Mutex},
    thread,
};

use block_downloader::block_downloader_thread_loop;
use peer_comunicator::worker_manager_loop;
use peer_comunicator::message_receiver_thread_loop;
use peer_comunicator::new_peer_conector_thread_loop;
use peer_comunicator::NEW_CONECTION_INTERVAL;

pub type FinishedIndicator = Arc<Mutex<bool>>;

pub enum Stops {
    GracefullStop,
    UngracefullStop,
    Continue,
}

impl Stops {
    pub fn log_message(&self, starting_message: String) -> String {
        match *self {
            Stops::GracefullStop => format!("{}: finished gracefully", starting_message),
            Stops::UngracefullStop => format!("{}: finished ungracefully", starting_message),
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
                    logger.log(Stops::GracefullStop.log_message(format!("Wroker {}", id)));
                    return Some(stream);
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(format!("Wroker {}", id)));
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
        if (stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()) || (stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()){
            logger.log(format!("Warning, could not set timeout for peer worker {}", id));
        }
        
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
                    logger.log(Stops::GracefullStop.log_message(format!("Wroker {}", id)));
                    return Some(stream);
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(format!("Wroker {}", id)));
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

#[derive(Debug)]
pub struct NewPeerConnector{
    thread: thread::JoinHandle<()>,
    new_workers_receiver: mpsc::Receiver<TcpStream>,
}

impl NewPeerConnector{
    /// Creates a worker for a MessageReceiver
    pub fn new(
        node_version: i32,
        node_address: SocketAddr, 
        logger: Logger, 
        finished: FinishedIndicator,
    ) -> Result<NewPeerConnector, PeerComunicatorError> {
        let listener = TcpListener::bind(node_address).map_err(|_| PeerComunicatorError::ErrorCantReceiveNewPeerConections)?;
        listener.set_nonblocking(true).map_err(|_| PeerComunicatorError::ErrorCantReceiveNewPeerConections)?;
        let (sender, receiver) = mpsc::channel();
        
        let thread = thread::spawn(move || loop {
            logger.log(format!("Peer connectorSigo vivo"));
            match new_peer_conector_thread_loop(
                &listener,
                node_version,
                node_address,
                &sender,
                &logger,
                &finished,
            ) {
                Stops::Continue => continue,
                Stops::GracefullStop => {
                    logger.log(Stops::GracefullStop.log_message("new peer conector".to_string()));
                    return;
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message("new peer connector".to_string()));
                    return;
                }
            }
        });

        Ok(NewPeerConnector { thread, new_workers_receiver: receiver })
    }

    pub fn recv_timeout(&self, timeout: Duration)->Result<TcpStream,RecvTimeoutError>{
        self.new_workers_receiver.recv_timeout(timeout)
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    pub fn join_thread(self) -> Result<(), WorkerError> {
        self.thread.join().map_err(|_| WorkerError::ErrorWorkerPaniced)
    }
}

#[derive(Debug)]
pub struct PeerComunicatorWorkerManager{
    thread: thread::JoinHandle<()>
}

impl PeerComunicatorWorkerManager{
    pub fn new(new_peer_conector: Option<NewPeerConnector>,
        mut workers: Vec<Worker>,
        safe_blockchain: SafeBlockChain,
        safe_headers: SafeVecHeader,
        safe_pending_tx: SafePendingTx,
        finished: Arc<Mutex<bool>>,
        logger: Logger)-> PeerComunicatorWorkerManager{
        let thread = thread::spawn(move || loop {
            match worker_manager_loop(
                &new_peer_conector,
                &mut workers,
                &safe_blockchain,
                &safe_headers,
                &safe_pending_tx,
                &finished,
                &logger) {
                Stops::Continue => continue,
                Stops::GracefullStop => {
                    logger.log(Stops::GracefullStop.log_message("peer communicator".to_string()));
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message("peer communicator".to_string()));
                }
            }
            
            //p
            if let Some(new_peer_conector) = new_peer_conector{
                if let Err(error) = new_peer_conector.join_thread(){
                    logger.log_error(&error);
                }
            } 
            for worker in workers{
                if let Err(error) = worker.join_thread(){
                    logger.log_error(&error);
                }
            }
            return;
        });
        PeerComunicatorWorkerManager { thread }
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    pub fn join_thread(self) -> Result<(), WorkerError> {
        self.thread.join().map_err(|_| WorkerError::ErrorWorkerPaniced)
}
}
