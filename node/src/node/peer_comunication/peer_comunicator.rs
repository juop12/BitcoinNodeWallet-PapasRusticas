use crate::{node::*, utils::btc_errors::PeerComunicatorError};

use std::{
    net::TcpStream,
    sync::{Arc, Mutex}, 
};

use workers::*;

#[derive(Debug)]
pub struct PeerComunicator {
    workers: Vec<Worker>,
    finished_working_indicator: FinishedIndicator,
    logger: Logger,
}

impl PeerComunicator {
    pub fn new(
        outbound_connections: &Vec<TcpStream>,
        safe_blockchain: &SafeBlockChain,
        safe_headers: &SafeVecHeader,
        safe_pending_tx: &SafePendingTx,
        logger: &Logger,
    ) -> PeerComunicator{
        let finished_working_indicator = Arc::new(Mutex::from(false));
        let workers = PeerComunicator::create_message_receivers(outbound_connections, safe_blockchain, safe_headers, safe_pending_tx, &finished_working_indicator, logger);
        PeerComunicator { 
            workers, 
            finished_working_indicator,
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
    pub fn finish_receiving(self) -> Result<(), PeerComunicatorError> {
        self.logger
            .log(String::from("Requested_end_of_comunications"));

        match self.finished_working_indicator.lock() {
            Ok(mut indicator) => *indicator = true,
            Err(_) => return Err(PeerComunicatorError::ErrorFinishingReceivingMessages),
        }

        for worker in self.workers {
            if let Err(error) = worker.join_thread() {
                self.logger.log_error(&error);
            }
        }

        self.logger.log(String::from("Disconected_from_peers"));
        Ok(())
    }
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

/// Main loop of eache message receiver
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

    if receive_message(
        stream,
        safe_block_headers,
        safe_block_chain,
        safe_pending_tx,
        logger,
        false,
    )
    .is_err()
    {
        return Stops::UngracefullStop;
    }
    Stops::Continue
}
