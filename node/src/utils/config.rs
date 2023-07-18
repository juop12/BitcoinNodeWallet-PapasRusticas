use super::btc_errors::ConfigError;
use chrono::{DateTime, Utc};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const CONFIG_FILENAME: &str = "nodo.conf";
const PARAMETER_AMOUNT: usize = 9;

/// Struct that represents a node's configuration parameters.
#[derive(Debug)]
pub struct Config {
    pub version: i32,
    pub dns_port: u16,
    pub local_host: [u8; 4],
    pub local_port: u16,
    pub log_path: String,
    pub begin_time: u32,
    pub headers_path: String,
    pub blocks_path: String,
    pub ipv6_enabled: bool,
}

impl Config {
    /// It validates the parameters sent as a String array to see if they can be used for a node's configuration.
    /// On error returns ErrorMismatchedQuantityOfParameters or ErrorMismatchedParameters depending on the circumstances.
    fn _validate_parameters(config_fields: &Vec<String>) -> Result<(), ConfigError> {
        if config_fields.len() != PARAMETER_AMOUNT {
            return Err(ConfigError::ErrorMismatchedQuantityOfParameters);
        }

        let begin_time: u32 = match parse_date(&config_fields[5]) {
            Some(time) => time,
            None => return Err(ConfigError::ErrorParsingDate),
        };

        if begin_time > (Utc::now().timestamp() as u32) {
            return Err(ConfigError::ErrorParsingDate);
        }

        Ok(())
    }

    /// It receives the fields for the configuration and returns a config with those values. In case of error returns None.
    fn _initialize(config_fields: &Vec<String>) -> Option<Config> {
        let mut local_host: [u8; 4] = [0; 4];
        let splitter = (*config_fields)[2].split(',');

        for (i, number) in (0_usize..).zip(splitter) {
            local_host[i] = number.parse::<u8>().ok()?;
        }

        let begin_time = parse_date(&config_fields[5])?;

        Some(Config {
            version: config_fields[0].parse::<i32>().ok()?,
            dns_port: config_fields[1].parse::<u16>().ok()?,
            local_host,
            local_port: config_fields[3].parse::<u16>().ok()?,
            log_path: config_fields[4].to_string(),
            begin_time,
            headers_path: config_fields[6].to_string(),
            blocks_path: config_fields[7].to_string(),
            ipv6_enabled: config_fields[8].parse::<bool>().ok()?,
        })
    }

    /// It receives the fields for the configuration, validates them and returns a Config if they were valid.
    /// On Error returns ErrorMismatchedQuantityOfParameters, ErrorMismatchedParameters or ErrorFillingAttributes depending on the circumstances.
    fn _from(config_fields: Vec<String>) -> Result<Config, ConfigError> {
        Config::_validate_parameters(&config_fields)?;

        match Config::_initialize(&config_fields) {
            Some(config) => Ok(config),
            None => Err(ConfigError::ErrorFillingAttributes),
        }
    }

    /// It receives a path to a file containing the fields for the configuration and returns a Config if both the path
    /// and the parameters were valid.
    /// On Error, it returns an error in the ConfigError enum.
    pub fn from_path(path: &str) -> Result<Config, ConfigError> {
        if !path.ends_with(CONFIG_FILENAME) {
            return Err(ConfigError::ErrorMismatchedFileName);
        }

        let file = _open_config_handler(path)?;
        let reader: BufReader<File> = BufReader::new(file);

        let mut config_fields = Vec::new();

        for line in reader.lines() {
            match line {
                Ok(field) => {
                    let splitted_line: Vec<&str> = field.split('=').collect();
                    config_fields.push(splitted_line[1].to_string());
                }
                Err(_) => return Err(ConfigError::ErrorReadingFile),
            }
        }

        Config::_from(config_fields)
    }
}

/// A handler for opening the file containing the config's attributes, on error returns ErrorReadingFile
fn _open_config_handler(path: &str) -> Result<File, ConfigError> {
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(_) => Err(ConfigError::ErrorReadingFile),
    }
}

/// It receives a string representing a date and returns its timestamp at 00:00:00 in case of success, None otherwise.
fn parse_date(line: &str) -> Option<u32> {
    let complete_date = format!("{}T00:00:00Z", line);

    let begin_time: u32 = match complete_date.parse::<DateTime<Utc>>() {
        Ok(datetime) => datetime.timestamp() as u32,
        Err(_) => return None,
    };

    Some(begin_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEGIN_TIME: &str = "2023-04-10";
    const LOG_FILE_PATH: &str = "tests_txt/config_test_log.txt";
    const HEADERS_FILE_PATH: &str = "tests_txt/headers.bin";
    const BLOCKS_FILE_PATH: &str = "tests_txt/blocks.bin";

    #[test]
    fn config_test_1_valid_file_creates_config() {
        assert!(Config::from_path("src/nodo.conf").is_ok());
    }

    #[test]
    fn config_test_2_invalid_file_cannot_create_config() {
        assert!(Config::from_path("invalid_file.conf").is_err());
    }

    #[test]
    fn config_test_3_saves_parameters_correctly() {
        let parameters = vec![
            "70015".to_string(),
            "18333".to_string(),
            "127,0,0,1".to_string(),
            "1001".to_string(),
            LOG_FILE_PATH.to_string(),
            BEGIN_TIME.to_string(),
            HEADERS_FILE_PATH.to_string(),
            BLOCKS_FILE_PATH.to_string(),
            false.to_string(),
        ];

        let expected_begin_time_timestamp: u32 = 1681084800;

        let config =
            Config::_from(parameters).expect("Could not create config from valid parameters.");

        assert_eq!(config.version, 70015);
        assert_eq!(config.dns_port, 18333);
        assert_eq!(config.local_host, [127, 0, 0, 1]);
        assert_eq!(config.local_port, 1001);
        assert_eq!(config.log_path, LOG_FILE_PATH.to_string());
        assert_eq!(config.begin_time, expected_begin_time_timestamp);
        assert_eq!(config.headers_path, HEADERS_FILE_PATH.to_string());
        assert_eq!(config.blocks_path, BLOCKS_FILE_PATH.to_string());
        assert_eq!(config.ipv6_enabled, false);
    }

    #[test]
    fn config_test_4_invalid_amount_parameters_cannot_create_config() {
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "127,0,0,1".to_string(),
            "1001".to_string(),
            BEGIN_TIME.to_string(),
            HEADERS_FILE_PATH.to_string(),
            BLOCKS_FILE_PATH.to_string(),
        ];

        assert!(Config::_from(parameters).is_err());
    }

    #[test]
    fn config_test_5_invalid_type_for_local_port_parameter_cannot_create_config() {
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "34".to_string(),
            "this should be a u16".to_string(),
            LOG_FILE_PATH.to_string(),
            BEGIN_TIME.to_string(),
            HEADERS_FILE_PATH.to_string(),
            BLOCKS_FILE_PATH.to_string(),
            true.to_string(),
        ];

        assert!(Config::_from(parameters).is_err());
    }

    #[test]
    fn config_test_6_future_begin_time_parameter_cannot_create_config() {
        let parameters = vec![
            "70015".to_string(),
            "53".to_string(),
            "34".to_string(),
            "this should be a u16".to_string(),
            LOG_FILE_PATH.to_string(),
            Utc::now().date_naive().succ_opt().unwrap().to_string(),
            HEADERS_FILE_PATH.to_string(),
            BLOCKS_FILE_PATH.to_string(),
            true.to_string(),
        ];

        assert!(Config::_from(parameters).is_err());
    }
}
