use crate::{
    messages::{
        get_data_message::*,
        message_trait::Message,
    },
    node::*,
    utils::btc_errors::BlockDownloaderError,
};

use std::{
    sync::{mpsc, Arc, Mutex},
    net::TcpStream,
    thread,
};


/// Struct that represents a worker thread in the thread pool.
#[derive(Debug)]
struct Worker {
    thread: thread::JoinHandle<bool>,
    stream: TcpStream,
    _id: usize,
}

type Bundle = Box<Vec<[u8;32]>>;

impl Worker {
    
    ///Creates a worker which attempts to execute tasks received trough the channel in a loop
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Bundle>>>, mut stream: TcpStream, locked_block_chain: Arc<Mutex<Vec<Block>>>, missed_bundles_sender: mpsc::Sender<Bundle>, logger: Logger) -> Result<Worker, BlockDownloaderError> {
        let stream_cpy = match stream.try_clone(){
            Ok(cloned_stream) => cloned_stream,
            Err(_) => return Err(BlockDownloaderError::ErrorCreatingWorker),
        };

        let thread = thread::spawn(move || loop {

            let stop = thread_loop(id, &receiver, &mut stream, &locked_block_chain, &missed_bundles_sender, &logger);
            
            if let Some(gracefull_stop) = stop {
                return gracefull_stop;
            }
        });
        
        Ok(Worker { _id : id, thread, stream: stream_cpy })
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    fn join_thread(self)->Result<Option<TcpStream>, BlockDownloaderError>{
        match self.thread.join(){
            Ok(clean) => {
                if clean{
                    return Ok(Some(self.stream));
                }
                Ok(None)
            },
            Err(_) => Err(BlockDownloaderError::ErrorWrokerPaniced),
        }
    }

    /// Returns true if the thread has finished its execution.
    fn is_finished(&self)->bool{
        self.thread.is_finished()
    }
}

/// Gets a bundle from the shared reference and logs the errors
fn get_bundle(id:usize, receiver: &Arc<Mutex<mpsc::Receiver<Bundle>>>, logger: &Logger) ->Option<Bundle>{
    let bundle = match receiver.lock() {
        Ok(rec_lock) => {
            match rec_lock.recv() {
                Ok(bundle) => bundle,
                Err(error) => {
                    logger.log(format!("Worker {id} failed: {:?}", error));
                    return None
                },
            }
        },
        Err(error) => {
            logger.log(format!("Worker {id} failed: {:?}", error));
            return None;
        },
    };
    Some(bundle)
}

/// Saves the blocks in the shared reference, if unable, loggs the erros and sends the bundle to the missed_bundles channel
fn save_blocks(id: usize, locked_block_chain: &Arc<Mutex<Vec<Block>>>, received_blocks: Vec<Block>, missed_bundles_sender: &mpsc::Sender<Bundle>, aux_bundle: Bundle,logger: &Logger) -> bool{
    match locked_block_chain.lock(){
        Ok(mut block_chain) => {
            for block in received_blocks{
                block_chain.push(block);
            }
            true
        },
        Err(error) => {
            if let Err(missed_error) = missed_bundles_sender.send(aux_bundle){
                logger.log(format!("Worker {id} failed sending missed bundle: {:?}", missed_error));
            };

            logger.log(format!("Worker {id} failed: {:?}", error));
            false
        }
    }
}

/// Main loop that worker's thread executes. It gets a bundle from the shared channel, 
/// gets the blocks from it's peer, and saves them to the shared reference block vector.
/// If anything fails along the way it loggs acordingly, as well as other things like 
/// received messages.
/// It returs an option representing wheter the loop must stop (Some) or continue None. 
/// The bool stored in Some represents whether the stop is a "Gracefull stop", meaning 
/// the thread must return true (for example when receiving an end of channel), or an 
/// "Ungracefull stop" meaning the loop failed at some point
fn thread_loop(id: usize, receiver: &Arc<Mutex<mpsc::Receiver<Bundle>>>, stream: &mut TcpStream, locked_block_chain: &Arc<Mutex<Vec<Block>>>, missed_bundles_sender: &mpsc::Sender<Bundle>, logger: &Logger) -> Option<bool>{
    let bundle = match get_bundle(id, &receiver, &logger){
        Some(bundle) => bundle,
        None => return Some(false),
    };

    //si se recibe un end_of_channel
    if bundle.is_empty(){
        return Some(true);
    }

    let aux_bundle = Box::new(*bundle.clone());
    
    let received_blocks = match get_blocks_from_bundle(*bundle, stream, &logger) {
        Ok(blocks) => blocks,
        Err(error) => {
            if let Err(error) = missed_bundles_sender.send(aux_bundle){
                logger.log(format!("Worker {id} failed sending missed bundle: {:?}", error));
            }
            
            if let BlockDownloaderError::BundleNotFound = error{
                logger.log(format!("Worker {id} did not find bundle"));
                return None;
            }else{
                logger.log(format!("Worker {id} failed: {:?}", error));
                return Some(false);
            }
        }
    };

    if !save_blocks(id, &locked_block_chain, received_blocks, &missed_bundles_sender, aux_bundle, &logger){
        return Some(false);
    }
    None
}

//=====================================================================================

#[derive(Debug)]
pub struct BlockDownloader{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Bundle>,
    missed_bundles_receiver: mpsc::Receiver<Bundle>,
    logger: Logger,
}

impl BlockDownloader{

    /// Creates a new thread pool with the specified size, it must be greater than zero.
    pub fn new(outbound_connections: &Vec<TcpStream>, header_stream_index: usize, block_chain :&Arc<Mutex<Vec<Block>>>, logger: Logger) -> Result<BlockDownloader, BlockDownloaderError>{
        let connections_ammount = outbound_connections.len();
        if connections_ammount == 0{
            return Err(BlockDownloaderError::ErrorInvalidCreationSize);
        }   
        
        let (sender, receiver) = mpsc::channel();
        let (missed_bundles_sender, missed_bundles_receiver) = mpsc::channel();
        
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(connections_ammount);
        
        //No tomamos el primer tcpStream, porque se usa para descargar headers.
        for id in 0..connections_ammount {
            if id == header_stream_index{
                continue;
            }
            
            let current_stream = match outbound_connections[id].try_clone(){
                Ok(stream) => stream,
                Err(_) => {
                    logger.log_error(&BlockDownloaderError::ErrorCreatingWorker);
                    continue;
                },
            };

            let new_worker = Worker::new(id, receiver.clone(), current_stream, block_chain.clone(), missed_bundles_sender.clone(), logger.clone());
            
            match new_worker{
                Ok(worker) => workers.push(worker),
                Err(_) => logger.log_error(&BlockDownloaderError::ErrorCreatingWorker),
            };
        }

        Ok(BlockDownloader {workers, sender, missed_bundles_receiver, logger})
    }
    
    /// Receives a function or closure that receives no parameters and executes them in a diferent thread using workers.x
    pub fn download_block_bundle(&self, bundle: Vec<[u8; 32]>) -> Result<(), BlockDownloaderError> {
        if bundle.is_empty(){
            return Ok(());
        }
        let box_bundle = Box::new(bundle);

        match self.sender.send(box_bundle){
            Ok(_) => Ok(()),
            Err(_err) => Err(BlockDownloaderError::ErrorSendingToThread),
        }
    }

    /// Writes an empty vector to the channel of the workers, so they can finish their execution. It works as
    /// a way to stop the threads execution. On error, it returns BlockDownloaderError.
    pub fn finish_downloading(&mut self)->Result<(), BlockDownloaderError>{
        let cantidad_workers = self.workers.len();
        let mut working_peer_conection = None;
        for _ in 0..cantidad_workers{
            let end_of_channel :Vec<[u8;32]> = Vec::new();
            if self.sender.send(Box::new(end_of_channel)).is_err(){
                self.logger.log(format!("Falló en el envio al end of channel"));
                return Err(BlockDownloaderError::ErrorSendingToThread);
            }

            while let Ok(bundle) = self.missed_bundles_receiver.try_recv(){
                self.download_block_bundle(*bundle)?;
            }
            
            let mut joined_a_worker = false;
            
            while !joined_a_worker{
                let worker = self.workers.remove(0);
                if worker.is_finished(){
                    let stream_op = worker.join_thread()?;
                    if working_peer_conection.is_none(){
                        working_peer_conection = stream_op;
                    };
                    joined_a_worker = true;
                }else{
                    self.workers.push(worker);
                }
            }
        }

        let mut stream = match working_peer_conection{
            Some(stream) => stream,
            None => return Err(BlockDownloaderError::ErrorAllWorkersFailed),
        };

        while let Ok(bundle) = self.missed_bundles_receiver.try_recv(){
            get_blocks_from_bundle(*bundle, &mut stream, &self.logger)?;
        }
        
        Ok(())
    }
}

/// Receives a TcpStream and gets the blocks from the stream, returning a BlockMessage.
fn receive_block_message(stream: &mut TcpStream, logger: &Logger) -> Result<BlockMessage, BlockDownloaderError>{
    let block_msg_h = match receive_message_header(stream){
        Ok(msg_h) => msg_h,
        Err(_) => return Err(BlockDownloaderError::ErrorReceivingBlockMessage),
    };

    logger.log(block_msg_h.get_command_name());

    let mut msg_bytes = vec![0; block_msg_h.get_payload_size() as usize];
    match stream.read_exact(&mut msg_bytes) {
        Err(_) => return Err(BlockDownloaderError::ErrorReceivingBlockMessage),
        Ok(_) => {}
    }

    let block_msg = match block_msg_h.get_command_name().as_str(){
        "block\0\0\0\0\0\0\0" => {
            match BlockMessage::from_bytes(&mut msg_bytes){
                Ok(block_msg) => block_msg,
                Err(_) => return Err(BlockDownloaderError::ErrorReceivingBlockMessage),
            }
        },
        "notfound\0\0\0\0" => return Err(BlockDownloaderError::BundleNotFound),
        _ => return receive_block_message(stream, logger),
    };

    Ok(block_msg)
}

/// Sends a getdata message to the stream, requesting the blocks with the specified hashes.
/// Returns an error if it was not possible to send the message.
fn send_get_data_message_for_blocks(hashes :Vec<[u8; 32]>, stream: &mut TcpStream)->Result<(), BlockDownloaderError>{
    
    let get_data_message = GetDataMessage::create_message_inventory_block_type(hashes);
    
    match get_data_message.send_to(stream) {
        Ok(_) => Ok(()),
        Err(_) => Err(BlockDownloaderError::ErrorSendingMessageBlockDownloader),
    }
}

/// Receives a vector of block hashes and a TcpStream, and returns a vector of blocks that were requested to the stream
pub fn get_blocks_from_bundle(requested_block_hashes: Vec<[u8;32]>, stream: &mut TcpStream, logger: &Logger)-> Result<Vec<Block>, BlockDownloaderError>{
    let amount_of_hashes = requested_block_hashes.len();
    send_get_data_message_for_blocks(requested_block_hashes, stream)?;
    let mut blocks :Vec<Block> = Vec::new();
    for _ in 0..amount_of_hashes{
        let received_message = receive_block_message(stream, logger)?;
        if validate_proof_of_work(&received_message.block.get_header()){
            if validate_proof_of_inclusion(&received_message.block){
                blocks.push(received_message.block);
            }else{
                logger.log(String::from("A block failed proof of inclusion"));
                return Err(BlockDownloaderError::ErrorValidatingBlock);
            }
        }else{
            logger.log(String::from("A block failed proof of work"));
            return Err(BlockDownloaderError::ErrorValidatingBlock);
        }
    }
    
    Ok(blocks)
}