use crate::{node::*, utils::{btc_errors::PeerComunicatorError, WorkerError}};

use std::{
    net::TcpStream,
    sync::{Arc, Mutex, mpsc::{RecvTimeoutError, self}}, 
};

use workers::*;

pub const NEW_CONECTION_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct PeerComunicator {
    //workers: Vec<Worker>,
    //new_peer_conector: WorkerManager,
    finished_working_indicator: FinishedIndicator,
    peer_communicator_manager: PeerComunicatorWorkerManager,
    logger: Logger,
}

impl PeerComunicator {
    pub fn new(
        node_version: i32,
        node_address: SocketAddr, 
        outbound_connections: &Vec<TcpStream>,
        safe_blockchain: &SafeBlockChain,
        safe_headers: &SafeVecHeader,
        safe_pending_tx: &SafePendingTx,
        logger: &Logger,
    ) -> PeerComunicator{
        let finished_working_indicator = Arc::new(Mutex::from(false));
        let workers = PeerComunicator::create_message_receivers(outbound_connections, safe_blockchain, safe_headers, safe_pending_tx, &finished_working_indicator, logger);
        let new_peer_conector = NewPeerConnector::new(
            node_version, 
            node_address, 
            logger.clone(), 
            finished_working_indicator.clone());
        if let Err(error) = &new_peer_conector{
            logger.log_error(error)
        }
        let worker_manager = PeerComunicatorWorkerManager::new(
            new_peer_conector.ok(), 
            workers, 
            safe_blockchain.clone(),
            safe_headers.clone(),
            safe_pending_tx.clone(),
            finished_working_indicator.clone(),
            logger.clone());

        PeerComunicator { 
            finished_working_indicator,
            peer_communicator_manager: worker_manager,
            logger: logger.clone()
        }
    }

    fn create_message_receivers(
        outbound_connections: &Vec<TcpStream>,
        safe_blockchain: &SafeBlockChain,
        safe_headers: &SafeVecHeader,
        safe_pending_tx: &SafePendingTx,
        finished_working_indicator: &Arc<Mutex<bool>>,
        logger: &Logger,
    ) -> Vec<Worker> {
        let amount_of_peers = outbound_connections.len();
        let mut workers = Vec::new();
        for (id, stream) in outbound_connections
            .iter()
            .enumerate()
            .take(amount_of_peers)
        {
            let current_stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => {
                    logger.log_error(&PeerComunicatorError::ErrorCreatingWorker);
                    continue;
                }
            };
            let worker = Worker::new_message_receiver_worker(
                current_stream,
                safe_headers.clone(),
                safe_blockchain.clone(),
                safe_pending_tx.clone(),
                logger.clone(),
                finished_working_indicator.clone(),
                id,
            );
            workers.push(worker);
        }
        workers
    }

    /// Joins all worker threads, trying to result in a gracefull finish
    pub fn end_of_communications(self) -> Result<(), PeerComunicatorError> {
        self.logger
            .log(String::from("Requested_end_of_comunications"));

        match self.finished_working_indicator.lock() {
            Ok(mut indicator) => *indicator = true,
            Err(_) => return Err(PeerComunicatorError::ErrorFinishingReceivingMessages),
        }

        if self.peer_communicator_manager.join_thread().is_err(){
            self.logger.log("worker manager paniced".to_string());
        };
        
        self.logger.log(String::from("Disconected_from_peers"));
        Ok(())
    }
}

pub fn worker_manager_loop(
    new_peer_connector: &Option<NewPeerConnector>,
    workers: &mut Vec<Worker>,
    safe_blockchain: &SafeBlockChain,
    safe_headers: &SafeVecHeader,
    safe_pending_tx: &SafePendingTx,
    finished: &Arc<Mutex<bool>>,
    logger: &Logger)-> Stops{
        
        match finished.lock() {
            Ok(finish) => {
                if *finish {
                    return Stops::GracefullStop;
                }
            }
            Err(_) => return Stops::UngracefullStop,

        }
        
        //recivir nuevos peers
        if let Some(new_peer_connector) = new_peer_connector{
            match new_peer_connector.recv_timeout(NEW_CONECTION_INTERVAL){
                Ok(new_stream) => {
                    let new_worker = Worker::new_message_receiver_worker(
                        new_stream,
                        safe_headers.clone(), 
                        safe_blockchain.clone(), 
                        safe_pending_tx.clone(), 
                        logger.clone(), 
                        finished.clone(), 
                        workers.len());
                    workers.push(new_worker);
                },
                Err(error) => if let RecvTimeoutError::Disconnected = error{
                    return Stops::GracefullStop
                },
            }
        }
        
        //sacar peers que hayan terminado
        let mut i = 0;
        while i < workers.len() {
            if workers[i].is_finished() {
                workers.swap_remove(i);
            } else {
                i += 1;
            }
        }
        
        Stops::Continue
        //firjarse de mandar mensajes
}


/// Main loop of each message receiver
pub fn message_receiver_thread_loop(
    stream: &mut TcpStream,
    safe_block_headers: &SafeVecHeader,
    safe_block_chain: &SafeBlockChain,
    safe_pending_tx: &SafePendingTx,
    logger: &Logger,
    finished: &FinishedIndicator,
) -> Stops {
    match finished.lock() {
        Ok(finish) => {
            if *finish {
                return Stops::GracefullStop;
            }
        }
        Err(_) => return Stops::UngracefullStop,
    }

    if let Err(error) = receive_message(
        stream,
        safe_block_headers,
        safe_block_chain,
        safe_pending_tx,
        logger,
        false,
    ){
        match error{
            NodeError::ErrorPeerTimeout => return Stops::Continue,
            _ => return Stops::UngracefullStop,
        }
    }
    Stops::Continue
}

pub fn new_peer_conector_thread_loop(
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
            logger.log("New peer requested conection".to_string());
            if let Err(error) = incoming_handshake(node_version, peer_address, node_address, &mut tcp_stream, &logger){
                logger.log("New peer failed handshake".to_string());
                return Stops::UngracefullStop;
            }
            if worker_sender.send(tcp_stream).is_err(){
                logger.log_error(&PeerComunicatorError::ErrorCantReceiveNewPeerConections);
                return Stops::UngracefullStop;
            };
            logger.log("New peer conection stablished".to_string());
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