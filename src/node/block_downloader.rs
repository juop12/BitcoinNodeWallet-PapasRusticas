use std::net::TcpStream;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use crate::messages::get_data_message::*;
use crate::messages::utils::Message;
use crate::node::*;
/// Struct that represents a worker thread in the thread pool.
#[derive(Debug)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
    stream: TcpStream,
}

type Bundle = Box<Vec<[u8;32]>>;

impl Worker {
    ///Creates a worker which attempts to execute tasks received trough the channel in a loop
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Bundle>>>, mut stream: TcpStream, locked_block_chain: Arc<Mutex<Vec<Block>>>, missed_bundles_sender: mpsc::Sender<Bundle>) -> Result<Worker, BlockDownloaderError> {
        let stream_cpy = match stream.try_clone(){
            Ok(cloned_stream) => cloned_stream,
            Err(_) => return Err(BlockDownloaderError::ErrorCreatingWorker),
        };

        let thread = thread::spawn(move || loop {
            let bundle = match receiver.lock() {
                Ok(rec_lock) => {
                    match rec_lock.recv() {
                        Ok(bundle) => bundle,
                        Err(_) => return,
                    }
                },
                Err(_) => return,
            };

            //si se recibe un vector vacio 
            if bundle.is_empty(){
                return
            }

            let a = *bundle.clone();
            
            let received_blocks = match get_blocks_from_bundle(*bundle, &mut stream) {
                Ok(blocks) => blocks,
                Err(_) => {
                    let _ = missed_bundles_sender.send(Box::new(a));
                    return;
                    }
            };

            match locked_block_chain.lock(){
                Ok(mut block_chain) => {
                    for block in received_blocks{
                        block_chain.push(block);
                    }
                },
                Err(_) => {
                    let _ = missed_bundles_sender.send(Box::new(a));
                    return;
                    }
            };
        });
        Ok(Worker { id, thread, stream: stream_cpy })
    }

    ///Joins the thread of the worker, returning an error if it was not possible to join it.
    fn join_thread(self)->Result<(), BlockDownloaderError>{
        match self.thread.join(){
            Ok(()) => Ok(()),
            Err(_) => Err(BlockDownloaderError::ErrorJoiningThread),
        }
    }

    fn is_finished(&self)->bool{
        self.thread.is_finished()
    }
}

/// Struct that represents a thread pool.
#[derive(Debug)]
pub struct BlockDownloader{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Bundle>,
    missed_bundles_receiver: mpsc::Receiver<Bundle>,
}

/// Enum that contains the possible errors that can occur when running the thread pool.
#[derive(Debug)]
pub enum BlockDownloaderError {
    ErrorInvalidCreationSize,
    ErrorSendingToThread,
    ErrorReceivingBlockMessage,
    ErrorSendingMessageBlockDownloader,
    ErrorCreatingWorker,
    ErrorJoiningThread,
}

impl BlockDownloader{
    /// Creates a new thread pool with the specified size, it must be greater than zero.
    pub fn new(out_bound_connections : &Vec<TcpStream>, block_chain :&Arc<Mutex<Vec<Block>>>)->Result<BlockDownloader, BlockDownloaderError>{
        let connections_ammount = out_bound_connections.len();
        if connections_ammount == 0{
            return Err(BlockDownloaderError::ErrorInvalidCreationSize);
        }   
        let (sender, receiver) = mpsc::channel();
        let (missed_bundles_sender, missed_bundles_receiver) = mpsc::channel();
        
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(connections_ammount);
        //No tomamos el primer tcpStream, porque se usa para descargar headers.
        for id in 1..connections_ammount {
            let current_stream = match out_bound_connections[id].try_clone(){
                Ok(stream) => stream,
                Err(_) => return Err(BlockDownloaderError::ErrorCreatingWorker),
            };
            let worker = Worker::new(id, receiver.clone(), current_stream, block_chain.clone(), missed_bundles_sender.clone())?;
            workers.push(worker); // la que estaba en The Rust Book es Arc::clone(&receiver)
        }
        Ok(BlockDownloader {workers, sender, missed_bundles_receiver})
    }
    
    ///Receives a function or closure that receives no parameters and executes them in a diferent thread using workers.x
    pub fn download_block_bundle(&self, bundle: Vec<[u8; 32]>) -> Result<(), BlockDownloaderError> {
        if bundle.is_empty(){
            return Ok(());
        }
        let box_bundle = Box::new(bundle);

        match self.sender.send(box_bundle) {
            Ok(_) => Ok(()),
            Err(_) => Err(BlockDownloaderError::ErrorSendingToThread),
        }
    }

    ///Writes an empty vector to the channel of the workers, so they can finish their execution. It works as
    ///a way to stop the threads execution. On error, it returns BlockDownloaderError.
    pub fn finish_downloading(&mut self)->Result<(), BlockDownloaderError>{
        let cantidad_workers = self.workers.len();
        for _ in 0..cantidad_workers{
            if self.sender.send(Box::new(vec![])).is_err() {
                return Err(BlockDownloaderError::ErrorSendingToThread);
            }

            while let Ok(bundle) = self.missed_bundles_receiver.try_recv(){
                //println!("represoesar {:?}", *bundle);
                self.download_block_bundle(*bundle)?;
            }

            let end_of_channel :Vec<[u8;32]> = Vec::new();
            if self.sender.send(Box::new(end_of_channel)).is_err(){
                return Err(BlockDownloaderError::ErrorSendingToThread);
            }
          
            
            let mut joined_a_worker = false;
            
            while !joined_a_worker{
                let worker = self.workers.remove(0);
                if worker.is_finished(){
                    worker.join_thread()?;
                    joined_a_worker = true;
                }else{
                    self.workers.push(worker);
                }
            }
        }
        Ok(())
    }

}

///Receives a TcpStream and gets the blocks from the stream, returning a BlockMessage.
fn receive_block_message(stream: &mut TcpStream) -> Result<BlockMessage, BlockDownloaderError>{
    let block_msg_h = match receive_message_header(stream){
        Ok(msg_h) => msg_h,
        Err(_) => return Err(BlockDownloaderError::ErrorReceivingBlockMessage),
    };
    
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
        "notfound\0\0\0\0" =>{
            println!("Hubo un not found");
            todo!();
        },
        _ => return receive_block_message(stream),
    };

    Ok(block_msg)

}

/// Sends a getdata message to the stream, requesting the blocks with the specified hashes.
/// Returns an error if it was not possible to send the message.
fn send_get_data_message_for_blocks(hashes :Vec<[u8; 32]>, stream: &mut TcpStream)->Result<(), BlockDownloaderError>{
    let count = vec![hashes.len() as u8];
    
    let get_data_message = GetDataMessage::create_message_inventory_block_type(hashes, count);
    
    match get_data_message.send_to(stream) {
        Ok(_) => Ok(()),
        Err(_) => Err(BlockDownloaderError::ErrorSendingMessageBlockDownloader),
    }
}

/// Receives a vector of block hashes and a TcpStream, and returns a vector of blocks that were requested to the stream
fn get_blocks_from_bundle(requested_block_hashes: Vec<[u8;32]>, stream: &mut TcpStream)-> Result<Vec<Block>, BlockDownloaderError>{
    let amount_of_hashes = requested_block_hashes.len();
    send_get_data_message_for_blocks(requested_block_hashes, stream)?;
    let mut blocks :Vec<Block> = Vec::new();
    for _ in 0..amount_of_hashes{
        let received_message = receive_block_message(stream)?;
        blocks.push(received_message.block);
    }
    
    Ok(blocks)
}