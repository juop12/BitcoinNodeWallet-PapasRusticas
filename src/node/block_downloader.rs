use std::net::TcpStream;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use crate::messages::get_data_message::*;
use crate::messages::utils::Message;
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Bundle>>>, stream: TcpStream) -> Worker {
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
            get_block_bundle(*bundle, &mut stream);
            //job();
        });

        Worker { id, thread, stream }
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
    ErrorSendingMessageBlockDownloader
}

impl BlockDownloader{
    /// Creates a new thread pool with the specified size, it must be greater than zero.
    fn new(out_bound_connections : Vec<TcpStream>)->Result<BlockDownloader, BlockDownloaderError>{
        let connections_ammount = out_bound_connections.len();
        if connections_ammount == 0{
            return Err(BlockDownloaderError::ErrorInvalidCreationSize);
        }   
        
        let (sender, receiver) = mpsc::channel();
        
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(connections_ammount);
        for id in 0..connections_ammount {
            workers.push(Worker::new(id,receiver.clone(), out_bound_connections[id])); // la que estaba en The Rust Book es Arc::clone(&receiver)
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

fn receive_message (stream: &mut TcpStream) -> Result<String, NodeError>{
    let block_headers_msg_h = self.receive_message_header(&mut stream)?;
    println!("\n\n{}", block_headers_msg_h.get_command_name());
    
    let mut msg_bytes = vec![0; block_headers_msg_h.get_payload_size() as usize];
    match stream.read_exact(&mut msg_bytes) {
        Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
        Ok(_) => {}
    }

    match block_headers_msg_h.get_command_name().as_str(){
        "headers\0\0\0\0\0" => self.handle_block_headers_message(msg_bytes, sync_node_index)?,
        "block\0\0\0\0\0\0\0" => self.handle_block_message(msg_bytes)?,
        _ => {},
    }

    Ok(block_headers_msg_h.get_command_name())

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

fn get_block_bundle(requested_block_hashes: Vec<[u8;32]>, stream: &mut TcpStream)-> Result<(), BlockDownloaderError>{
    println!("\n\nentre a block bundle");
    let amount_of_hashes = requested_block_hashes.len();
    send_get_data_message_for_blocks(requested_block_hashes, stream)?;
    for _ in 0..amount_of_hashes{
        let mut received_message_type = receive_block_message(0)?;
        println!("no es el primer receive");
        while (received_message_type != "block\0\0\0\0\0\0\0") && (received_message_type != "notfound\0\0\0\0"){
            received_message_type = receive_block_message(0)?;
        }
    }
    //mandar al nodo los bloques obtenidos
    Ok(())
}