use crate::{node::*, utils::{btc_errors::BlockDownloaderError, log}};

use std::{
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use block_downloader::block_downloader_thread_loop;
use peer_comunicator::message_receiver_thread_loop;

use super::peer_comunicator::SafeWorkers;
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

struct WorkerManager{
    thread: thread::JoinHandle<()>,
}

impl WorkerManager{
    /// Creates a worker for a MessageReceiver
    pub fn new(
        node_version: i32,
        node_address: SocketAddr, 
        safe_block_headers: SafeVecHeader,
        safe_blockchain: SafeBlockChain,
        safe_pending_tx: SafePendingTx,
        logger: Logger, 
        finished: FinishedIndicator,
        safe_workers: SafeWorkers
    ) -> Result<WorkerManager, NodeError> {
        let listener = TcpListener::bind(node_address).map_err(|_| NodeError::ErrorCantReceiveNewPeerConections)?;
        listener.set_nonblocking(true).map_err(|_| NodeError::ErrorCantReceiveNewPeerConections)?;
        let id = 0;
        let thread = thread::spawn(move || loop {
            logger.log(format!("Sigo vivo: {}", id));
            match peer_conection_manager_thread_loop(
                &listener,
                node_version,
                node_address,
                &safe_block_headers,
                &safe_blockchain,
                &safe_pending_tx,
                &logger,
                &finished,
                &safe_workers,
            ) {
                Stops::Continue => continue,
                Stops::GracefullStop => {
                    logger.log(Stops::GracefullStop.log_message(id));
                }
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(id));
                }
            }
        });

        Ok(WorkerManager { thread })
    }
}

fn peer_conection_manager_thread_loop(
    listener: &TcpListener,
    node_version: i32,
    node_address: SocketAddr, 
    safe_block_headers: &SafeVecHeader,
    safe_blockchain: &SafeBlockChain,
    safe_pending_tx: &SafePendingTx,
    logger: &Logger, 
    finished: &FinishedIndicator,
    safe_workers: &SafeWorkers)-> Stops{

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
            match safe_workers.lock() {
                Ok(workers) => {
                    if let Some(a) = {
                        Worker::new_message_receiver_worker(
                            tcp_stream,
                            safe_block_headers.clone(), 
                            safe_blockchain.clone(), 
                            safe_pending_tx.clone(), 
                            logger.clone(), 
                            finished.clone(), 
                            workers.len());

                    }
                }
                Err(_) => return Stops::UngracefullStop,
            }
        },
        Err(error) => {
            if error.kind() == std::io::ErrorKind::WouldBlock{
                sleep(NEW_CONECTION_INTERVAL);
                Stops::Continue
            }else{
                return Stops::UngracefullStop
            }
        },
    }
}
