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

pub struct Logger<T>{
    tx: Sender<T>, 
}

impl Logger<&str> {

    pub fn from_path(path: &str) -> Result<Logger<&'static str>, LoggerError> {

        let mut file = _open_config_handler(path)?;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            loop {
                let received: &str = match rx.recv(){
                    Ok(msg) => msg,
                    Err(_) => continue, // Waits for another msg.
                };

                if let Err(_) = file.write(received.as_bytes()){
                    continue; // Waits for another msg.
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

    fn log<T: BTCError>(&self, error: &'static T) -> Result<(), LoggerError>{
        let message = error.decode();  
        if let Err(_) = self.tx.send(message){
            return Err(LoggerError::ErrorSendingMessage);
        };

        Ok(())
    }
}

pub trait BTCError{

    fn decode(&self) -> &str;
}

/// A handler for opening the log file in write mode, on error returns ErrorOpeningFile
fn _open_config_handler(path: &str) -> Result<File, LoggerError> {
    match File::create(path){
        Ok(file)=> Ok(file),
        Err(_) => Err(LoggerError::ErrorOpeningFile),
    }
}