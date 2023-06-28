use glib::{Continue, Sender as GlibSender};
use gtk::prelude::*;
use gtk::{Application, Builder, Label, Window};
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

pub enum LoadingSreenError {
    ErrorReadingFile,
    ErrorReadingMetadataFromFile,
    ErrorSeekingFile,
    ErrorReadingLine,
}

const LINES_SHOWN: usize = 11;
const SENDER_ERROR: &str = "Error sending message to node through mpsc channel";
const REFRESH_LOGIN_SCREEN_TIME: Duration = Duration::from_secs(1);

/// Reads the last lines of a File and returns them as a
/// vector of Strings where the least recent
/// line is the last element of the vector.
/// On error returns a LoadingScreenError
fn read_last_lines(file_path: &str, num_lines: usize) -> Result<Vec<String>, LoadingSreenError> {
    let file = File::open(file_path).map_err(|_| LoadingSreenError::ErrorReadingFile)?;
    let file_size = file
        .metadata()
        .map_err(|_| LoadingSreenError::ErrorReadingMetadataFromFile)?
        .len();
    let mut reader = BufReader::new(file);

    let mut lines: Vec<String> = Vec::new();
    let mut buffer: Vec<u8> = vec![0; 1024]; // Tamaño del buffer ajustable según tus necesidades
    let mut offset = file_size;
    let mut remaining_lines = num_lines;

    while remaining_lines > 0 && offset > 0 {
        let read_bytes = if offset < buffer.len() as u64 {
            offset as usize
        } else {
            buffer.len()
        };

        offset -= read_bytes as u64;

        reader
            .seek(SeekFrom::Start(offset))
            .map_err(|_| LoadingSreenError::ErrorSeekingFile)?;
        reader
            .read_exact(&mut buffer[..read_bytes])
            .map_err(|_| LoadingSreenError::ErrorReadingLine)?;

        let mut line_start = read_bytes;
        for (i, &byte) in buffer[..read_bytes].iter().enumerate().rev() {
            if byte == b'\n' {
                remaining_lines -= 1;
                if remaining_lines == 0 {
                    line_start = i + 1;
                    break;
                }
            }
        }

        let buffer_lines = String::from_utf8_lossy(&buffer[line_start..read_bytes]);
        let new_lines: Vec<String> = buffer_lines.lines().map(|line| line.to_owned()).collect();
        lines.extend(new_lines);
    }

    lines.reverse();

    Ok(lines)
}

/// Shows the loading screen and starts a thread that reads the node log file
/// and updates the loading screen every second with the last lines of the log file
/// for the user to see the progress of the initialization of the node.
pub fn show_loading_screen(builder: &Builder, app: &Application, running: Arc<Mutex<bool>>) {
    let loading_window: Window = builder.object("Loading Screen Window").unwrap();
    loading_window.set_title("Loading Screen");
    loading_window.set_application(Some(app));
    loading_window.show_all();
    let log_label: Label = builder.object("Log Label").unwrap();
    let (sx, rx) = glib::MainContext::channel::<Vec<String>>(glib::PRIORITY_DEFAULT);
    thread::spawn(move || obtain_loading_progress(sx, running));
    rx.attach(None, move |mut contents| {
        contents.reverse();
        let result: String = contents
            .iter()
            .map(|s| s.replace('\0', ""))
            .collect::<Vec<String>>()
            .join("\n");
        log_label.set_text(&result);

        Continue(true)
    });
}

fn send_last_lines_to_login_screen(sender: GlibSender<Vec<String>>) {
    if let Ok(contents) = read_last_lines("./src/node_log.txt", LINES_SHOWN) {
        if !contents.is_empty() {
            sender.send(contents).expect(SENDER_ERROR);
        }
    } else {
        println!("No se pudo leer el archivo");
    }
}

/// Reads the node log file and sends the last lines to the UI thread
fn obtain_loading_progress(sender: GlibSender<Vec<String>>, running: Arc<Mutex<bool>>) {
    loop {
        thread::sleep(REFRESH_LOGIN_SCREEN_TIME);
        match running.lock() {
            Ok(program_running) => {
                if !*program_running {
                    send_last_lines_to_login_screen(sender.clone());
                } else {
                    break;
                }
            }
            Err(_) => return,
        }
    }
}
