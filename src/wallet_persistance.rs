use gtk::{
    prelude::*, 
    ComboBoxText
}; 
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use crate::UiError;


const PATH_NAME: &str = "src/wallets.csv"; 

pub fn save_wallet_in_disk(priv_key: &str, name_text: &str) -> Result<(), UiError>{    
    
    let mut file = _open_write_handler(PATH_NAME)?;

    if writeln!(file, "{},{}", priv_key,name_text).is_err() {
        return Err(UiError::ErrorWritingFile);
    }

    if file.flush().is_err() {
        return Err(UiError::ErrorWritingFile);
    };

    Ok(())
}

pub fn get_saved_wallets_from_disk(wallet_selector: &ComboBoxText) -> Result<(), UiError>{

    let file = _open_read_only_handler(PATH_NAME)?;
    let reader: BufReader<File> = BufReader::new(file);

    let mut count = 0;

    for line in reader.lines() {
        let field = line.map_err(|_| UiError::ErrorReadingFile)?;

        let splitted_line: Vec<&str> = field.split(',').collect();
        wallet_selector.append(Some(&splitted_line[0]),splitted_line[1]);

        count += 1;
    }

    if count == 0{
        return Err(UiError::WalletsCSVWasEmpty);
    }
        
    Ok(())
}

fn _open_write_handler(path: &str) -> Result<File, UiError>{
    OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .map_err(|_| UiError::ErrorWritingFile)
}

fn _open_read_only_handler(path: &str) -> Result<File, UiError> {
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(error) =>{
            //if let  = error {

            //}
            Err(UiError::ErrorReadingFile)   
        },
    }
}