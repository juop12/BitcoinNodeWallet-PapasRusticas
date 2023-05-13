use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
/// Struct that represents a worker thread in the thread pool.
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

type Job =  Box<dyn FnOnce() + Send + 'static> ;

impl Worker {
    ///Creates a worker which attempts to execute tasks received trough the channel in a loop
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker { id, thread }
    }
}

/// Struct that represents a thread pool.
struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

/// Enum that contains the possible errors that can occur when running the thread pool.
enum ThreadPoolError {
    ErrorInvalidCreationSize,
    ErrorSendingToThread,
}

impl ThreadPool{
    /// Creates a new thread pool with the specified size, it must be greater than zero.
    fn new(size: usize)->Result<ThreadPool, ThreadPoolError>{
            
        if size == 0{
            return Err(ThreadPoolError::ErrorInvalidCreationSize);
        }
        
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id,receiver.clone())); // la que estaba en The Rust Book es Arc::clone(&receiver)
        }
        Ok(ThreadPool {workers, sender})
    }
    
    ///Receives a function or closure that receives no parameters and executes them in a diferent thread using workers.x
    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<(), ThreadPoolError> {
        let job = Box::new(f);

        match self.sender.send(job) {
            Ok(_) => Ok(()),
            Err(_) => Err(ThreadPoolError::ErrorSendingToThread),
        }
    }
}

