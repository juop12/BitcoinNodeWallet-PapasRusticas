use std::sync::{mpsc, mpsc::Sender};
use std::fs::OpenOptions;
use std::io::Write;
use std::fs::File;
use std::thread;

//use crate::messages::Message;

/// Struct that represents errors that can occur with the log.
#[derive(Debug)]
pub enum LoggerError{
    ErrorOpeningFile,
    ErrorLoggingMessage,
}

/// This struct has the responsability to write to a file.
pub struct Logger{
    tx: Sender<String>, 
}

impl Logger {

    /// Creates a new logger from a path, on error returns ErrorOpeningFile.
    pub fn from_path(path: &str) -> Result<Logger, LoggerError> {

        let mut file = _open_log_handler(path)?;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            loop {
                let received: String = match rx.recv(){
                    Ok(msg) => msg,
                    Err(_) => continue,
                };

                if let Err(_) = write!(file, "{}\n", received){
                    continue;
                };

                if let Err(_) = file.flush(){
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

    /*
        pub fn log_error<T: BTCError>(&self, error: &T) -> Result<(), LoggerError>{
            let text = error.decode();  
            self.log(text)
        }
    */

    /*
        pub fn log_message<T: Message>(&self, message: T) -> Result<(), LoggerError>{
            let text = message.decode();  
            self.log(text)
        }
    */

    /// Writes a text to the log, on error returns ErrorLoggingMessage.
    pub fn log(&self, text: String) -> Result<(), LoggerError>{
        if let Err(_) = self.tx.send(text){
            return Err(LoggerError::ErrorLoggingMessage);
        };

        Ok(())
    }

}

/*
    pub trait BTCError{
        fn decode(&self) -> String;
    }
*/

/// A handler for opening the log file in write mode, on error returns ErrorOpeningFile
fn _open_log_handler(path: &str) -> Result<File, LoggerError> {
    match OpenOptions::new().create(true).write(true).open(path){
        Ok(file)=> {
            Ok(file)
        },
        Err(_) => Err(LoggerError::ErrorOpeningFile),
    }
}

/* 
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, BufRead};

    const LOGFILE: &str = "test_log.txt";
    const STOP: &str = "stop";


    /* 
    #[test]
    fn peer_discovery_test_1_fails_when_receiving_invalid_dns_address() {


    }
    */
    /*
    #[test]
    fn log_test_1_writes_text_correctly() {
        let logger = Logger::from_path(LOGFILE).unwrap();

        logger.log("prueba".to_string()).unwrap();
        logger.log(STOP.to_string()).unwrap();

        let file = File::open(LOGFILE).unwrap();
        let contenido: Vec<String> = BufReader::new(file).lines().flatten().collect();

        assert!((contenido[0] == "prueba") && (contenido[1] == "stop"));
    }
    */
}
*/