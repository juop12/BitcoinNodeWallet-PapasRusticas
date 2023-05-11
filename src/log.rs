use std::sync::{mpsc, mpsc::Sender};
use std::io::Write;
use std::fs::File;
use std::thread;

/// Struct that represents errors that can occur with the log.
#[derive(Debug)]
pub enum LoggerError{
    ErrorOpeningFile,
    ErrorSendingMessage,
}

pub struct Logger{
    tx: Sender<String>, 
}

impl Logger {

    pub fn from_path(path: &str) -> Result<Logger, LoggerError> {

        let mut file = _open_log_handler(path)?;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            loop {
                let received: String = match rx.recv(){
                    Ok(msg) => msg,
                    Err(_) => continue,
                };

                if let Err(_) = file.write(received.as_bytes()){
                    continue;
                };

                if received == "stop"{
                    break;
                }
            }
        });

        Ok(Logger {
            tx,
        })
    }

    fn log<T: BTCError>(&self, error: &T) -> Result<(), LoggerError>{
        let message = error.decode();  
        if let Err(_) = self.tx.send(message){
            return Err(LoggerError::ErrorSendingMessage);
        };

        Ok(())
    }
}

pub trait BTCError{
    fn decode(&self) -> String;
}

/// A handler for opening the log file in write mode, on error returns ErrorOpeningFile
fn _open_log_handler(path: &str) -> Result<File, LoggerError> {
    match File::create(path){
        Ok(file)=> Ok(file),
        Err(_) => Err(LoggerError::ErrorOpeningFile),
    }
}