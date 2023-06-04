use crate::{
    node::*,
    utils::{btc_errors::MessageReceiverError},
};

use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
};

pub type SafeBlockChain = Arc<Mutex<HashMap<[u8; 32], Block>>>;
pub type SafeVecHeader = Arc<Mutex<Vec<BlockHeader>>>;
pub type FinishedIndicator = Arc<Mutex<bool>>;

enum Stops{
    GracefullStop,
    UngracefullStop,
    Continue,
}

impl Stops{
    fn log_message(&self, id: usize)->String{
        match *self{
            Stops::GracefullStop => format!("Worker {} finished gracefully", id),
            Stops::UngracefullStop => format!("Worker {} finished ungracefully", id),
            Stops::Continue => String::new(),
        }
        
    }
}

#[derive(Debug)]
struct Worker{
    thread: thread::JoinHandle<Option<TcpStream>>,
    _id: usize,
}

impl Worker{
    fn new(mut stream: TcpStream, safe_block_headers: SafeVecHeader, safe_block_chain: SafeBlockChain, logger: Logger, finished: FinishedIndicator, id: usize)->Worker{

        let thread = thread::spawn(move || loop {
            logger.log(format!("Sigo vivo: {}", id));
            match thread_loop(&mut stream, &safe_block_headers, &safe_block_chain, &logger, &finished){
                Stops::GracefullStop => {
                    logger.log(Stops::GracefullStop.log_message(id));
                    return Some(stream);
                },
                Stops::UngracefullStop => {
                    logger.log(Stops::UngracefullStop.log_message(id));
                    return None
                },
                Stops::Continue => continue,
            }

        });

        Worker{ thread, _id: id}
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    fn join_thread(self) -> Result<Option<TcpStream>, MessageReceiverError> {
        match self.thread.join() {
            Ok(stream) => Ok(stream),
            Err(_) => Err(MessageReceiverError::ErrorWrokerPaniced),
        }
    }
}

fn insert_time_orderly(header: BlockHeader, vec_headers: &mut Vec<BlockHeader>){
    let mut i = vec_headers.len();
    while (i > 0) && (header.time < vec_headers[i-1].time){
        i-=1;
    }
    vec_headers.insert(i, header);
}

///Adds a block to the blockchain, its header to the headers vector and saves them both on disk.
fn add_block_or_headers(mut received_block_headers: Vec<BlockHeader>, received_blocks: Vec<Block>, safe_headers: & SafeVecHeader, safe_blockchain: & SafeBlockChain, logger: &Logger) -> Result<(), MessageReceiverError> {
    if !received_blocks.is_empty(){
        match safe_blockchain.lock(){
            Ok(mut blockchain) => {
                for block in received_blocks{
                    if !blockchain.contains_key(&block.header_hash()){
                        received_block_headers.push(block.get_header().to_owned());
                        blockchain.insert(block.header_hash(), block);
                        logger.log(String::from("Se almaceno un nuevo bloque"))
                    }
                }
            },
            Err(_) => return Err(MessageReceiverError::ErrorAddingReceivedData),
        }
    }
    
    if !received_block_headers.is_empty(){
        match safe_headers.lock(){
            Ok(mut headers) => {
                for header in received_block_headers{
                    insert_time_orderly(header, &mut headers);
                    logger.log(String::from("Se almaceno un nuevo header"));
                };
            },
            Err(_) => return Err(MessageReceiverError::ErrorAddingReceivedData),
        }
    }
    Ok(())

}

fn thread_loop(stream: &mut TcpStream, safe_block_headers: &SafeVecHeader, safe_block_chain: &SafeBlockChain, logger: &Logger, finished: &FinishedIndicator)-> Stops{

    match finished.lock(){
        Ok(finish) => {
            if *finish{
                return Stops::GracefullStop;
            }
        },
        Err(_) => return Stops::UngracefullStop,
        
    }

    let mut aux_blockchain = Vec::new();
    let mut aux_block_headers = Vec::new();
    if receive_message(stream, &mut aux_block_headers, &mut aux_blockchain, &logger, false).is_err(){
        return Stops::UngracefullStop;
    }

    if let Err(error) = add_block_or_headers(aux_block_headers, aux_blockchain, safe_block_headers, safe_block_chain, logger){
        logger.log_error(&error);
        return Stops::UngracefullStop;
    }
    Stops::Continue
}

#[derive(Debug)]
pub struct MessageReceiver {
    workers: Vec<Worker>,
    finished_working_indicators: Vec<FinishedIndicator>,
    logger: Logger,
}

impl MessageReceiver{
    pub fn new(outbound_connections: Vec<TcpStream>, safe_blockchain: SafeBlockChain, safe_headers: SafeVecHeader, logger: &Logger)->MessageReceiver{
        let amount_of_peers = outbound_connections.len();
        let mut workers = Vec::new();
        let mut finished_working_indicators = Vec::new();
        for (id, stream) in outbound_connections
        .into_iter()
        .enumerate()
        .take(amount_of_peers)
        {
            let finished = Arc::new(Mutex::from(false));
            let worker = Worker::new(stream, safe_headers.clone(), safe_blockchain.clone(), logger.clone(), finished.clone(), id);
            workers.push(worker);
            finished_working_indicators.push(finished);
        };
        MessageReceiver { workers, finished_working_indicators, logger: logger.clone()}

    }

    pub fn finish_receiving(self)-> Result<(), MessageReceiverError>{
        self.logger.log(String::from("Requested_end_of_comunications"));
        for indicator in self.finished_working_indicators{
            match indicator.lock(){
                Ok(mut indicator) => *indicator = true,
                Err(_) => {
                    self.logger.log_error(&MessageReceiverError::ErrorFinishingReceivingMessages);
                    return Err(MessageReceiverError::ErrorFinishingReceivingMessages);
                },
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