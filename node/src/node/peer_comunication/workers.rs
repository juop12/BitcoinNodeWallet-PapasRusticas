use crate::{node::*, utils::{btc_errors::BlockDownloaderError, log, PeerComunicatorError, WorkerError}};

use std::{
    net::TcpStream,
    sync::{mpsc::{self, RecvTimeoutError}, Arc, Mutex},
    thread,
};

use block_downloader::block_downloader_thread_loop;
use peer_comunicator::message_receiver_thread_loop;
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

    /*
fn listen_new_conections(&self){
    let listener = TcpListener::bind(self.address).unwrap();
    if listener.set_nonblocking(true).is_err(){
        self.logger.log_error(&NodeError::ErrorCantReceiveNewPeerConections);
        return;
    };
    let logger = self.logger.clone();
    let version = self.version;
    let node_address = self.address;

    let thread = thread::spawn(move || loop {
        match listener.accept(){
            Ok((mut tcp_stream, peer_address)) => {
                match incoming_handshake(version, peer_address, node_address, &mut tcp_stream, &logger){
                    Ok(_) => {

                    },
                    Err(error) => return Err(error),
                }
            },
            Err(error) => {
                if error.kind() == std::io::ErrorKind::WouldBlock{
                    sleep(NEW_CONECTION_INTERVAL);
                }
            },
        }
    }
);
}
*/

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

fn new_peer_conector_thread_loop(
    listener: &TcpListener,
    node_version: i32,
    node_address: SocketAddr, 
    worker_sender: &mpsc::Sender<TcpStream>,
    logger: &Logger, 
    finished: &FinishedIndicator)-> Stops{

    match finished.lock() {
        Ok(finish) => {
            if *finish {
                return Stops::GracefullStop;
            }
        }
        Err(_) => return Stops::UngracefullStop,
    }

    match listener.accept(){
        Ok((mut tcp_stream, peer_address)) => {
            if let Err(error) = incoming_handshake(node_version, peer_address, node_address, &mut tcp_stream, &logger){
                return Stops::UngracefullStop
            }
            /*
            let new_worker = Worker::new_message_receiver_worker(
                tcp_stream,
                safe_block_headers.clone(), 
                safe_blockchain.clone(), 
                safe_pending_tx.clone(), 
                logger.clone(), 
                finished.clone(), 
                *worker_id);
            *worker_id +=1;
            */
            if worker_sender.send(tcp_stream).is_err(){
                logger.log_error(&PeerComunicatorError::ErrorCantReceiveNewPeerConections);
                return Stops::UngracefullStop;
            };
        },
        Err(error) => {
            if error.kind() == std::io::ErrorKind::WouldBlock{
                sleep(NEW_CONECTION_INTERVAL);
            }else{
                return Stops::UngracefullStop;
            }
        },
    }
    Stops::Continue
}
