use std::{io::{BufRead, BufReader}, fs::File};

const CONFIG_FILENAME : &str = "nodo.conf";
const PARAMETER_AMOUNT : usize = 5;
//const CURRENT_VERSION : i32 = 70015;

/*
// nodo.conf

    version=70015
    dns_port=53
    local_host=127,0,0,1
    local_port=1001
    log_file_path=path/to/log_file
*/

#[derive(Debug)]
pub enum ConfigError{
    ErrorReadingFile,
    ErrorFillingAttributes,
    ErrorMismatchedFileName,
    ErrorMismatchedQuantityOfParameters,
    ErrorMismatchedParameters,
}

#[derive(Debug)]
pub struct Config {
    version: i32,
    dns_port: u16,
    local_host: [u8; 4],
    local_port: u16,
    log_file_path: String,
}

impl Config{
    fn _validate_parameters(config_fields: &Vec<String>) -> Result<(), ConfigError>{
        if config_fields.len() != PARAMETER_AMOUNT {
            return Err(ConfigError::ErrorMismatchedQuantityOfParameters);        
        }
        
        /*  
        if let Some(version) = config_fields[0].parse::<i32>().ok(){
            if version != CURRENT_VERSION {
                return Err(ConfigError::ErrorMismatchedParameters);
            }
        }
        */

        // match config_fields[0].parse::<i32>().ok() {
        //     Some(version) => {
        //         if version != CURRENT_VERSION {
        //             return Err(ConfigError::ErrorMismatchedParameters);
        //         }
        //     },
        //     None => return Err(ConfigError::ErrorMismatchedParameters),
        // };
        
        Ok(())
    }
    
    fn _initialize(config_fields: &Vec<String>)->Option<Config>{
        let mut local_host: [u8; 4] = [0; 4];
        let splitter = (*config_fields)[2].split(',');
        
        for (i, number) in (0_usize..).zip(splitter){
            local_host[i] = number.parse::<u8>().ok()?;
        }
        

        Some(Config {
            version: config_fields[0].parse::<i32>().ok()?,
            dns_port: config_fields[1].parse::<u16>().ok()?,
            local_host, 
            local_port: config_fields[3].parse::<u16>().ok()? ,
            log_file_path: config_fields[4].to_string(),
        })
    }

    fn _from(config_fields: Vec<String>) -> Result<Config, ConfigError>{

        Config::_validate_parameters(&config_fields)?;

        match Config::_initialize(&config_fields) {
            Some(config) => Ok(config),
            None => Err(ConfigError::ErrorFillingAttributes),
        }
    }

    pub fn from_path(path: &str) -> Result<Config, ConfigError>{
        if !path.ends_with(CONFIG_FILENAME){
            return Err(ConfigError::ErrorMismatchedFileName)
        }

        let file = _open_config_handler(path)?;
        let reader: BufReader<File> = BufReader::new(file);

        let mut config_fields = Vec::new();
                
        for line in reader.lines() {
            match line {
                Ok(field) => {
                    let splitted_line: Vec<&str> = field.split('=').collect();
                    config_fields.push(splitted_line[1].to_string());
                },
                Err(_) => return Err(ConfigError::ErrorReadingFile),
            }
        }
        
        Config::_from(config_fields)
    }
}

fn _open_config_handler(path: &str) -> Result<File, ConfigError> {
    match File::open(path){
        Ok(file)=> Ok(file),
        Err(_) => Err(ConfigError::ErrorReadingFile),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_1_valid_file_creates_config(){
        assert!(Config::from_path("src/nodo.conf").is_ok());
    }

    #[test]
    fn test_config_2_invalid_file_cannot_create_config(){
        assert!(Config::from_path("invalid_file.conf").is_err());
    } 

    #[test]
    fn test_config_3_saves_parameters_correctly(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "127,0,0,1".to_string(),
            "1001".to_string(),
            "path/to/log_file".to_string(),
        ];
        
        let config = Config::_from(parameters).expect("Could not create config from valid parameters.");
        
        assert_eq!(config.version, 70015);
        assert_eq!(config.dns_port, 53);
        assert_eq!(config.local_host, [127,0,0,1]);
        assert_eq!(config.local_port, 1001);
        assert_eq!(config.log_file_path, "path/to/log_file".to_string());
    }

    #[test]
    fn test_config_4_invalid_ammount_parameters_cannot_create_config(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "127,0,0,1".to_string(),
            "1001".to_string(),
        ];
        
        assert!(Config::_from(parameters).is_err());
    }
    
    #[test]
    fn test_config_5_invalid_type_of_parameters_cannot_create_config(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "34".to_string(),
            "this should be a u16".to_string(),
            "path/to/log_file".to_string(),
        ];
        
        assert!(Config::_from(parameters).is_err());
    }        
}

