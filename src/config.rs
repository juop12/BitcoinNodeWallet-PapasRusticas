use std::{io::{BufRead, BufReader}, fs::File, path::Path};

const CONFIG_FILENAME : &str = "nodo.conf";
const PARAMETER_AMOUNT : usize = 5;
//const CURRENT_VERSION : i32 = 70015;

/// Struct that represents errors that can occur with the config setup.
#[derive(Debug)]
pub enum ConfigError{
    ErrorReadingFile,
    ErrorFillingAttributes,
    ErrorMismatchedFileName,
    ErrorMismatchedQuantityOfParameters,
    ErrorMismatchedParameters,
}


/// Struct that represents a node's configuration parameters.
#[derive(Debug)]
pub struct Config {
    pub version: i32,
    pub dns_port: u16,
    pub local_host: [u8; 4],
    pub local_port: u16,
    pub log_path: String,
}

impl Config{
    /// It validates the parameters sent as a String array to see if they can be used for a node's configuration.
    /// On error returns ErrorMismatchedQuantityOfParameters or ErrorMismatchedParameters depending on the circumstances.
    fn _validate_parameters(config_fields: &Vec<String>) -> Result<(), ConfigError>{
        if config_fields.len() != PARAMETER_AMOUNT {
            return Err(ConfigError::ErrorMismatchedQuantityOfParameters);        
        }

        let path = Path::new(config_fields[4].as_str());
        if !path.is_file() {
            return Err(ConfigError::ErrorMismatchedParameters);
        }

        /*  
        if let Some(version) = config_fields[0].parse::<i32>().ok(){
            if version != CURRENT_VERSION {
                return Err(ConfigError::ErrorMismatchedParameters);
            }
        }

        match config_fields[0].parse::<i32>().ok() {
            Some(version) => {
                if version != CURRENT_VERSION {
                    return Err(ConfigError::ErrorMismatchedParameters);
                }
            },
            None => return Err(ConfigError::ErrorMismatchedParameters),
        };
        */

        Ok(())
    }
    
    /// It receives the fields for the configuration and returns a config with those values. In case of error returns None.
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
            log_path: config_fields[4].to_string(),
        })
    }

    /// It receives the fields for the configuration, validates them and returns a Config if they were valid.
    /// On Error returns ErrorMismatchedQuantityOfParameters, ErrorMismatchedParameters or ErrorFillingAttributes depending on the circumstances.
    fn _from(config_fields: Vec<String>) -> Result<Config, ConfigError>{

        Config::_validate_parameters(&config_fields)?;

        match Config::_initialize(&config_fields) {
            Some(config) => Ok(config),
            None => Err(ConfigError::ErrorFillingAttributes),
        }
    }

    /// It receives a path to a file containing the fields for the configuration and returns a Config if both the path 
    /// and the parameters were valid.
    /// On Error, it returns an error in the ConfigError enum.
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

/// A handler for opening the file containing the config's attributes, on error returns ErrorReadingFile
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
            "18333".to_string(),
            "127,0,0,1".to_string(),
            "1001".to_string(),
            "src/node_log.txt".to_string(),
        ];
        
        let config = Config::_from(parameters).expect("Could not create config from valid parameters.");
        
        assert_eq!(config.version, 70015);
        assert_eq!(config.dns_port, 18333);
        assert_eq!(config.local_host, [127,0,0,1]);
        assert_eq!(config.local_port, 1001);
        assert_eq!(config.log_path, "src/node_log.txt".to_string());
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
    fn test_config_5_invalid_type_for_local_port_parameter_cannot_create_config(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "34".to_string(),
            "this should be a u16".to_string(),
            "src/node_log.txt".to_string(),
        ];
        
        assert!(Config::_from(parameters).is_err());
    }

    #[test]
    fn test_config_6_log_file_not_found_from_log_path_parameter_cannot_create_config(){
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "34".to_string(),
            "this should be a u16".to_string(),
            "src/node_log.txt".to_string(),
        ];
        
        assert!(Config::_from(parameters).is_err());
    }      
}

