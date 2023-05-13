use std::net::TcpStream;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use crate::messages::get_data_message::*;
use crate::messages::utils::Message;
use crate::node::*;
/// Struct that represents a worker thread in the thread pool.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
    stream: TcpStream,
}

type Job =  Box<dyn FnOnce() + Send + 'static> ;
type Bundle = Box<Vec<[u8;32]>>;

impl Worker {
    ///Creates a worker which attempts to execute tasks received trough the channel in a loop
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Bundle>>>, mut stream: TcpStream, locked_block_chain: Arc<Mutex<Vec<Block>>>) -> Result<Worker, BlockDownloaderError> {
        let mut stream_cpy = match stream.try_clone(){
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
            
            println!("Worker {id} got a job; executing.");
            let received_blocks = match get_blocks_from_bundle(*bundle, &mut stream) {
                Ok(blocks) => blocks,
                Err(_) => return,
            };

            let mut block_chain = match locked_block_chain.lock(){
                Ok(mut block_chain) => {
                    for block in received_blocks{
                        block_chain.push(block);
                    }
                },
                Err(_) => return,
            };
        });
        Ok(Worker { id, thread, stream: stream_cpy })
    }

    
}

/// Struct that represents a thread pool.
struct BlockDownloader{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Bundle>,
}

/// Enum that contains the possible errors that can occur when running the thread pool.
enum BlockDownloaderError {
    ErrorInvalidCreationSize,
    ErrorSendingToThread,
    ErrorReceivingBlockMessage,
    ErrorSendingMessageBlockDownloader,
    ErrorCreatingWorker,
}

impl BlockDownloader{
    /// Creates a new thread pool with the specified size, it must be greater than zero.
    fn new(mut out_bound_connections : Vec<TcpStream>, block_chain :Arc<Mutex<Vec<Block>>>)->Result<BlockDownloader, BlockDownloaderError>{
        let connections_ammount = out_bound_connections.len();
        if connections_ammount == 0{
            return Err(BlockDownloaderError::ErrorInvalidCreationSize);
        }   
        let (sender, receiver) = mpsc::channel();
        
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(connections_ammount);
        for id in 0..connections_ammount {
            let current_stream = match out_bound_connections[id].try_clone(){
                Ok(stream) => stream,
                Err(_) => return Err(BlockDownloaderError::ErrorCreatingWorker),
            };
            let worker = Worker::new(id,receiver.clone(), current_stream, block_chain.clone())?;
            workers.push(worker); // la que estaba en The Rust Book es Arc::clone(&receiver)
        }
        Ok(BlockDownloader {workers, sender})
    }
    
    ///Receives a function or closure that receives no parameters and executes them in a diferent thread using workers.x
    pub fn download_block_bundle(&self, bundle: Vec<[u8; 32]>) -> Result<(), BlockDownloaderError> {
        let box_bundle = Box::new(bundle);

        match self.sender.send(box_bundle) {
            Ok(_) => Ok(()),
            Err(_) => Err(BlockDownloaderError::ErrorSendingToThread),
        }
    }
}

fn receive_block_message(stream: &mut TcpStream) -> Result<BlockMessage, BlockDownloaderError>{
    let mut stream_cpy = match stream.try_clone() {
        Ok(stream_cpy) => stream_cpy,
        Err(_) => return Err(BlockDownloaderError::ErrorReceivingBlockMessage),
    };
    let block_msg_h = match receive_message_header(stream_cpy){
        Ok(msg_h) => msg_h,
        Err(_) => return Err(BlockDownloaderError::ErrorReceivingBlockMessage),
    };
    println!("\n\n{}", block_msg_h.get_command_name());
    
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
        _ => return receive_block_message(stream),
    };

    Ok(block_msg)

}

fn send_get_data_message_for_blocks(hashes :Vec<[u8; 32]>, stream: &mut TcpStream)->Result<(), BlockDownloaderError>{
    let count = vec![hashes.len() as u8];
    
    let get_data_message = GetDataMessage::create_message_inventory_block_type(hashes, count);
    println!("{:?}", get_data_message);
    
    match get_data_message.send_to(stream) {
        Ok(_) => Ok(()),
        Err(_) => Err(BlockDownloaderError::ErrorSendingMessageBlockDownloader),
    }
}

fn get_blocks_from_bundle(requested_block_hashes: Vec<[u8;32]>, stream: &mut TcpStream)-> Result<Vec<Block>, BlockDownloaderError>{
    println!("\n\nentre a block bundle");
    let amount_of_hashes = requested_block_hashes.len();
    send_get_data_message_for_blocks(requested_block_hashes, stream)?;
    let mut blocks :Vec<Block> = Vec::new();
    for _ in 0..amount_of_hashes{
        let received_message = receive_block_message(stream)?;
        blocks.push(received_message.block);
    }
    //mandar al nodo los bloques obtenidos
    Ok(blocks)
}