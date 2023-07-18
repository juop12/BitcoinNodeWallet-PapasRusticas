use super::BtcError;
use chrono::Utc;
use std::{
    io::Write,
    sync::{mpsc, mpsc::Sender},
    thread,
    {fs, fs::File, fs::OpenOptions},
};

const LOGGER_KILLER: &str = "stop";

/// Struct that represents errors that can occur with the log.
#[derive(Debug)]
pub enum LoggerError {
    ErrorOpeningFile,
    ErrorLoggingMessage,
}

/// This struct has the responsability to write to a file.
#[derive(Debug, Clone)]
pub struct Logger {
    tx: Sender<String>,
}

impl Logger {
    /// Creates a new logger from a path, on error returns ErrorOpeningFile.
    pub fn from_path(path: &str) -> Result<Logger, LoggerError> {
        let mut file = _open_log_handler(path)?;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || loop {
            let received: String = match rx.recv() {
                Ok(msg) => msg,
                Err(_) => continue,
            };

            if writeln!(file, "{}: {}", Utc::now(), received).is_err() {
                continue;
            };

            if file.flush().is_err() {
                continue;
            };

            if received == LOGGER_KILLER {
                break;
            }
        });

        Ok(Logger { tx })
    }

    /// Writes an error as text to the log, nothing happens on error.
    pub fn log_error<T: BtcError>(&self, error: &T) {
        self.log(error.to_string())
    }

    /// Writes a text to the log, nothing happens on error.
    pub fn log(&self, text: String) {
        // If the logger fails, we don't want to stop the program
        _ = self.tx.send(text);
    }
}

/// A handler for opening the log file in write mode, on error returns ErrorOpeningFile
fn _open_log_handler(path: &str) -> Result<File, LoggerError> {
    _ = fs::remove_file(path);

    match OpenOptions::new().create(true).write(true).open(path) {
        Ok(file) => Ok(file),
        Err(_) => Err(LoggerError::ErrorOpeningFile),
    }
}
