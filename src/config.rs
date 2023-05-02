use std::{io::{BufRead, BufReader}, fs::File};

const PARAMETER_AMOUNT : usize = 6;

/*
// nodo.conf

    version=70015
    dns_port=53
    local_host=127,0,0,1
    local_port=1001
    message_header_size=24
    log_file_path=path/to/log_file
*/

enum ConfigError{
    ErrorReadingFile,
    ErrorFillingAttributes,
    ErrorMismatchedParameters,
}


struct Config {
    version: i32,
    dns_port: u16,
    local_host: [u8; 4],
    local_port: u16,
    message_header_size: usize,
    log_file_path: String,
}

impl Config{
    fn validate_parameters(lines: Vec<&str>) -> Result<(), ConfigError>{
        todo!();
    }
    
    fn initialize(lines: Vec<&str>)->Option<Config>{
        let mut local_host: [u8; 4] = [0; 4];
        
        //lines[2].split(',').map(f)

        let mut j = 0;
        for i in lines[2].split(','){
            local_host[j] = i.parse::<u8>().ok()?;
            j += 1;
        };
        

        Some(Config {
            version: lines[0].parse::<i32>().ok()?,
            dns_port: lines[1].parse::<u16>().ok()?,
            local_host, 
            local_port: lines[3].parse::<u16>().ok()? ,
            message_header_size: lines[4].parse::<usize>().ok()?,
            log_file_path: lines[5].to_string(),
        })
    }

    fn from(lines: Vec<&str>) -> Result<Config, ConfigError>{

        Config::validate_parameters(lines)?;
        let config = Config::initialize(lines);

        match config {
            Some(config) => Ok(config),
            None => Err(ConfigError::ErrorFillingAttributes),
        }
    }

    pub fn from_path(path: &str) -> Result<Config, ConfigError>{
        let file = open_config_handler(path)?;
        let reader: BufReader<File> = BufReader::new(file);
        
        let mut fields = Vec::new();
                
        for read_line in reader.lines() {
            match read_line {
                Ok(line) => {
                    let mut splitter = line.splitn(2, '='); 
                    match splitter.nth(1){
                        Some(second_half) => fields.push(second_half),
                        None => return Err(ConfigError::ErrorReadingFile),
                    }
                },
                Err(_) => return Err(ConfigError::ErrorReadingFile),
            }
        }
        
        Config::from(fields)
    }
}

fn open_config_handler(path: &str) -> Result<File, ConfigError> {
    match File::open(path){
        Ok(file)=> Ok(file),
        Err(_) => Err(ConfigError::ErrorReadingFile),
    }
}

