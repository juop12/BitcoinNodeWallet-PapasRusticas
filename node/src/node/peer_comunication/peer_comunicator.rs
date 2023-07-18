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

///The PeerCommunicator is responsible for handeling all incomming messages, sending any message to the peers and accepting new peer conections
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
        let workers = PeerComunicator::create_peer_comunicator_workers(outbound_connections, safe_blockchain, safe_headers, safe_pending_tx, &finished_working_indicator, logger);
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

    ///Creates a PeerCommunicatorWorker for each stream, making each of them responsible for communicating with their corresponding peer 
    fn create_peer_comunicator_workers(
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
            let worker = Worker::new_peer_comunicator_worker(
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

    ///sends the given bytes to all currently connected peers
    pub fn send_message<T: Message>(&self, message: &T)->Result<(), PeerComunicatorError>{
        self.peer_communicator_manager.send_message(message)
    }
}

///Main loop for the worker manager, attemps to create a new worker from any new connection the NewPeerConnector
///might have stablished. Checks if there are any messages to send to the net, and joins any trhead corresponding 
///to a worker that already finished. 
pub fn worker_manager_loop(
    new_peer_connector: &Option<NewPeerConnector>,
    workers: &mut Vec<Worker>,
    safe_blockchain: &SafeBlockChain,
    safe_headers: &SafeVecHeader,
    safe_pending_tx: &SafePendingTx,
    message_bytes_receiver: &mpsc::Receiver<Vec<u8>>,
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
                    let new_worker = Worker::new_peer_comunicator_worker(
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
        
        match message_bytes_receiver.try_recv() {
            Ok(message_bytes) => {
                //sacar peers que hayan terminado
                let mut i = 0;
                let mut message_sent = false;
                while i < workers.len() {
                    if workers[i].is_finished() {
                        let removed_worker = workers.swap_remove(i);
                        if let Err(error) = removed_worker.join_thread(){
                            logger.log_error(&error);
                        }
                    } else {
                        if workers[i].send_message_bytes(message_bytes.clone()).is_ok(){
                            message_sent = true;
                        };
                        i += 1;
                    }
                }
                if !message_sent{
                    return Stops::UngracefullStop;
                }
            }
            Err(mpsc::TryRecvError::Empty) => {},
            _ => return Stops::UngracefullStop,
        };
        
        Stops::Continue
        //firjarse de mandar mensajes
}


/// Main loop for each peer communicator worker, attemps to receive a message form its peer and handles it.
/// If there is a message to send then it sends it to its peer
pub fn peer_comunicator_worker_thread_loop(
    stream: &mut TcpStream,
    safe_block_headers: &SafeVecHeader,
    safe_block_chain: &SafeBlockChain,
    safe_pending_tx: &SafePendingTx,
    message_bytes_receiver: &mpsc::Receiver<Vec<u8>>,
    logger: &Logger,
    finished: &FinishedIndicator,
    id: usize,
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

    match message_bytes_receiver.try_recv(){
        Ok(message_bytes) => {
            if stream.write_all(&message_bytes).is_err(){
                logger.log_error(&PeerComunicatorError::ErrorSendingMessage);
                return Stops::UngracefullStop;
            }
            logger.log(format!("Mandado mensaje al peer: {id}"));
        },
        Err(mpsc::TryRecvError::Empty) => {},
        _ => {
            logger.log_error(&WorkerError::LostConnectionToManager);
            return Stops::UngracefullStop
        }
    };
    Stops::Continue
}

/// Checks for new incomming connections, if a successfull handshake is done then it sends the new TcpStream to
/// the worker manager in orther to make a new PeerConnectoWorker to communicate with the new peer. 
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
            if incoming_handshake(node_version, peer_address, node_address, &mut tcp_stream, &logger).is_err(){
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