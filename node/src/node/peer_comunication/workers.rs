use crate::{node::*, utils::{PeerComunicatorError, WorkerError}};

use std::{
    net::TcpStream,
    sync::{mpsc::{self, RecvTimeoutError}, Arc, Mutex},
    thread,
};

use block_downloader::block_downloader_thread_loop;
use peer_comunicator::worker_manager_loop;
use peer_comunicator::peer_comunicator_worker_thread_loop;
use peer_comunicator::new_peer_conector_thread_loop;

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
    message_bytes_sender: Option<mpsc::Sender<Vec<u8>>>,
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

        if (stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()) || (stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()){
            logger.log(format!("Warning, could not set timeout for peer worker {}", id));
        }

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
                    logger.log(Stops::GracefullStop.log_message(format!("Worker {}", id)));
                    return Some(stream);
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(format!("Worker {}", id)));
                    return None;
                }
                Stops::Continue => continue,
            }
        });

        Worker {thread, message_bytes_sender: None, _id: id }
    }
    
    /// Creates a worker responsible for communicating with a peer
    pub fn new_peer_comunicator_worker(
        mut stream: TcpStream,
        safe_block_headers: SafeVecHeader,
        safe_block_chain: SafeBlockChain,
        safe_pending_tx: SafePendingTx,
        safe_headers_index: SafeHeaderIndex,
        logger: Logger,
        finished: FinishedIndicator,
        id: usize,
    ) -> Worker {
        if (stream.set_write_timeout(Some(PEER_TIMEOUT)).is_err()) || (stream.set_read_timeout(Some(PEER_TIMEOUT)).is_err()){
            logger.log(format!("Warning, could not set timeout for peer worker {}", id));
        }

        let (message_bytes_sender, message_bytes_receiver) = mpsc::channel();

        let thread = thread::spawn(move || loop {
            logger.log(format!("Worker {} continues execution", id));
            match peer_comunicator_worker_thread_loop(
                &mut stream,
                &safe_block_headers,
                &safe_block_chain,
                &safe_pending_tx,
                &safe_headers_index,
                &message_bytes_receiver,
                &logger,
                &finished,
                id,
            ) {
                Stops::GracefullStop => {
                    logger.log(Stops::GracefullStop.log_message(format!("Worker {}", id)));
                    return Some(stream);
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(format!("Worker {}", id)));
                    return None;
                }
                Stops::Continue => continue,
            }
        });

        Worker {thread, message_bytes_sender: Some(message_bytes_sender), _id: id }
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    pub fn join_thread(self) -> Result<Option<TcpStream>, WorkerError> {
        match self.thread.join() {
            Ok(stream) => Ok(stream),
            Err(_) => Err(WorkerError::ErrorWorkerPanicked),
        }
    }

    /// Returns true if the thread has finished its execution.
    pub fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }

    //sends the given bytes to the workers corresponding peer
    pub fn send_message_bytes(&self, message_bytes: Vec<u8>)-> Result<(), PeerComunicatorError>{
        match &self.message_bytes_sender{
            Some(sender) => sender.send(message_bytes).map_err(|_| PeerComunicatorError::ErrorSendingMessage),
            None => Err(PeerComunicatorError::ErrorSendingMessage),
        }
    }
}

#[derive(Debug)]
pub struct NewPeerConnector{
    thread: thread::JoinHandle<()>,
    new_workers_receiver: mpsc::Receiver<TcpStream>,
}

impl NewPeerConnector{
    /// Creates a worker responsible for receiving incoming connections from new peers
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
            logger.log(format!("Peer connector continues execution"));
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

    ///Receives a new connection, if no new connection is received whithin the duration, then it times out
    pub fn recv_timeout(&self, timeout: Duration)->Result<TcpStream,RecvTimeoutError>{
        self.new_workers_receiver.recv_timeout(timeout)
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    pub fn join_thread(self) -> Result<(), WorkerError> {
        self.thread.join().map_err(|_| WorkerError::ErrorWorkerPanicked)
    }
}

#[derive(Debug)]
pub struct PeerComunicatorWorkerManager{
    thread: thread::JoinHandle<()>,
    message_bytes_sender: mpsc::Sender<Vec<u8>>
}

impl PeerComunicatorWorkerManager{
    /// Creates a worker responsible for managing all the other peer communicator workers
    pub fn new(new_peer_conector: Option<NewPeerConnector>,
        mut workers: Vec<Worker>,
        safe_blockchain: SafeBlockChain,
        safe_headers: SafeVecHeader,
        safe_pending_tx: SafePendingTx,
        safe_headers_index: SafeHeaderIndex,
        finished: Arc<Mutex<bool>>,
        logger: Logger)-> PeerComunicatorWorkerManager{
        
        let (message_bytes_sender,message_bytes_receiver) = mpsc::channel();

        let thread = thread::spawn(move || loop {
            match worker_manager_loop(
                &new_peer_conector,
                &mut workers,
                &safe_blockchain,
                &safe_headers,
                &safe_pending_tx,
                &safe_headers_index,
                &message_bytes_receiver,
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
        PeerComunicatorWorkerManager { thread, message_bytes_sender }
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    pub fn join_thread(self) -> Result<(), WorkerError> {
        self.thread.join().map_err(|_| WorkerError::ErrorWorkerPanicked)
    }

    //Sends a message to all of the workers so they can then send them to their corresponding peers
    pub fn send_message<T: MessageTrait>(&self, message: &T)-> Result<(), PeerComunicatorError>{
        let mut message_bytes  = message.get_header_message().map_err(|_| PeerComunicatorError::ErrorSendingMessage)?.to_bytes();
        message_bytes.extend(message.to_bytes());
        self.message_bytes_sender.send(message_bytes).map_err(|_| PeerComunicatorError::ErrorSendingMessage)
    }
}
