use crate::{
    node::*,
    utils::btc_errors::MessageReceiverError,
};

use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};
use workers::*;

#[derive(Debug)]
pub struct MessageReceiver {
    workers: Vec<Worker>,
    finished_working_indicators: Vec<FinishedIndicator>,
    logger: Logger,
}

impl MessageReceiver{
    pub fn new(outbound_connections: &Vec<TcpStream>, safe_blockchain: &SafeBlockChain, safe_headers: &SafeVecHeader, safe_pending_tx: &SafePendingTx, logger: &Logger)->MessageReceiver{
        let amount_of_peers = outbound_connections.len();
        let mut workers = Vec::new();
        let mut finished_working_indicators = Vec::new();
        for (id, stream) in outbound_connections
        .iter()
        .enumerate()
        .take(amount_of_peers)
        {
            let current_stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(_) => {
                    logger.log_error(&MessageReceiverError::ErrorCreatingWorker);
                    continue;
                }
            };
            let finished = Arc::new(Mutex::from(false));
            let worker = Worker::new_message_receiver_worker(current_stream, safe_headers.clone(), safe_blockchain.clone(), safe_pending_tx.clone(), logger.clone(), finished.clone(), id);
            workers.push(worker);
            finished_working_indicators.push(finished);
        };
        MessageReceiver { workers, finished_working_indicators, logger: logger.clone()}

    }

    /// Joins all worker threads, trying to result in a gracefull finish
    pub fn finish_receiving(self)-> Result<(), MessageReceiverError>{
        self.logger.log(String::from("Requested_end_of_comunications"));

        for indicator in self.finished_working_indicators{
            match indicator.lock(){
                Ok(mut indicator) => *indicator = true,
                Err(_) =>return Err(MessageReceiverError::ErrorFinishingReceivingMessages),
            }
        }

        for worker in self.workers{
            if let Err(error) = worker.join_thread(){
                self.logger.log_error(&error);
            }
        }
        
        self.logger.log(String::from("Disconected_from_peers"));
        Ok(())
    }
}

/// Main loop of eache message receiver
pub fn message_receiver_thread_loop(stream: &mut TcpStream, 
    safe_block_headers: &SafeVecHeader, 
    safe_block_chain: &SafeBlockChain, 
    safe_pending_tx: &SafePendingTx,
    logger: &Logger, 
    finished: &FinishedIndicator
)-> Stops{

    match finished.lock(){
        Ok(finish) => {
            if *finish{
                return Stops::GracefullStop;
            }
        },
        Err(_) => return Stops::UngracefullStop,
        
    }

    if receive_message(stream, safe_block_headers, safe_block_chain, safe_pending_tx, logger, false).is_err(){
        return Stops::UngracefullStop;
    }
    Stops::Continue
}

