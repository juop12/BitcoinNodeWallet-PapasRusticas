use std::{io::{BufRead, BufReader}, fs::File};

const PARAMETER_AMOUNT : usize = 6;
const CURRENT_VERSION : i32 = 70015;
/*
// nodo.conf

    version=70015
    dns_port=53
    local_host=127,0,0,1
    local_port=1001
    message_header_size=24
    log_file_path=path/to/log_file
*/

#[derive(Debug)]
enum ConfigError{
    ErrorReadingFile,
    ErrorFillingAttributes,
    ErrorMismatchedParameters,
}

#[derive(Debug)]
struct Config {
    version: i32,
    dns_port: u16,
    local_host: [u8; 4],
    local_port: u16,
    message_header_size: usize,
    log_file_path: String,
}

impl Config{
    fn validate_parameters(lines: &Vec<String>) -> Result<(), ConfigError>{
    if lines.len() != PARAMETER_AMOUNT {
        return Err(ConfigError::ErrorMismatchedParameters);        
    }
    match lines[0].parse::<i32>().ok() {
        Some(version) => {
            if version != CURRENT_VERSION {
                return Err(ConfigError::ErrorMismatchedParameters);
            }
        },
        None => return Err(ConfigError::ErrorMismatchedParameters),
    };
    Ok(())
    }
    
    fn initialize(lines: &Vec<String>)->Option<Config>{
        let mut local_host: [u8; 4] = [0; 4];
        
        //lines[2].split(',').map(f)

        let mut j = 0;
        for i in (*lines)[2].split(','){
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

    fn from(lines: Vec<String>) -> Result<Config, ConfigError>{

        Config::validate_parameters(&lines)?;
        let config = Config::initialize(&lines);

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
                    if let Some(second_half) = splitter.nth(1) {
                        fields.push(second_half.to_string()); // Crear una copia de la segunda mitad de la línea
                    } else {
                        return Err(ConfigError::ErrorReadingFile);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_valid_parameters_creates_valid_config(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "127,0,0,1".to_string(),
            "1001".to_string(),
            "24".to_string(),
            "path/to/log_file".to_string(),
        ];
        assert!(Config::from(parameters).is_ok());
    }

    #[test]
    fn test_2_invalid_ammount_parameters_cannot_create_config(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "127,0,0,1".to_string(),
            "1001".to_string(),
            "24".to_string(),
        ];
        assert!(Config::from(parameters).is_err());
    }

    #[test]
    fn test_3_invalid_type_of_parameters_cannot_create_config(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "34,92,33,1".to_string(),
            "2023".to_string(),
            "path/to/log_file".to_string(),
            "24".to_string(),
        ];
        assert!(Config::from(parameters).is_err());
    }


    #[test]
    fn test_config_4_invalid_file_cannot_create_config(){
        assert!(Config::from_path("invalid_file.conf").is_err());
    }

    #[test]
    fn test_config_5_valid_file_creates_config(){
        assert!(Config::from_path("nodo.conf").is_ok());
    }

}

