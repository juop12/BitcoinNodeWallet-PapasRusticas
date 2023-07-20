use super::btc_errors::ConfigError;
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

const VERSION: &str = "version";
const LOCAL_ADDRES: &str = "local_address";
const BEGIN_TIME: &str = "starting_date";
const LOG_PATH: &str = "log_file_path";
const HEADERS_PATH: &str = "headers_file_path";
const BLOCKS_PATH: &str = "blocks_file_path";
const IPV6_ENABLED: &str = "ipv6_enabled";
const DNS: &str = "DNS";
const EXTERNAL_ADDR: &str = "external_addr";

const CONFIG_FILENAME: &str = "nodo.conf";
const PARAMETER_AMOUNT: usize = 9;

const IP_DELIMETER: char = ',';
const PORT_DELIMETER: char = ':';
const ARRAY_DELIMETER: char = ';';

/// Struct that represents a node's configuration parameters.
#[derive(Debug)]
pub struct Config {
    pub version: i32,
    pub local_address: ([u8; 4], u16),
    pub begin_time: u32,
    pub log_path: String,
    pub headers_path: String,
    pub blocks_path: String,
    pub ipv6_enabled: bool,
    pub dns: Vec<(String, u16)>,
    pub external_addresses: Vec<([u8; 4], u16)>,
}

impl Config {

    /// It receives a path to a file containing the fields for the configuration and returns a Config if both the path
    /// and the parameters were valid.
    /// On Error, it returns an error in the ConfigError enum.
    pub fn from_path(path: &str) -> Result<Config, ConfigError> {
        if !path.ends_with(CONFIG_FILENAME) {
            return Err(ConfigError::ErrorMismatchedFileName);
        }

        let file = _open_config_handler(path)?;
        let reader: BufReader<File> = BufReader::new(file);

        let mut config_fields = HashMap::new();

        for line in reader.lines() {
            match line {
                Ok(field) => {
                    let splitted_line: Vec<&str> = field.split('=').collect();
                    config_fields.insert(splitted_line[0].to_string(), splitted_line[1].to_string());
                }
                Err(_) => return Err(ConfigError::ErrorReadingFile),
            }
        }

        Config::_from(config_fields)
    }

    /// It receives the fields for the configuration, validates them and returns a Config if they were valid.
    /// Returns a ConfigError when parsing failed or a parameter is invalid. 
    fn _from(config_fields: HashMap<String, String>) -> Result<Config, ConfigError> {
        if config_fields.len() != PARAMETER_AMOUNT {
            return Err(ConfigError::ErrorMismatchedQuantityOfParameters);
        }

        Config::_initialize(config_fields)
    }

    /// It receives the fields for the configuration and returns a config 
    /// with those values. In case of error returns None.
    fn _initialize(config_fields: HashMap<String, String>) -> Result<Config, ConfigError> {
        let version = parse_version(&get_handler(&config_fields, VERSION)?)?;
        let local_address = parse_address(&get_handler(&config_fields, LOCAL_ADDRES)?)?;
        let begin_time = Config::_parse_date(&get_handler(&config_fields, BEGIN_TIME)?)?;
        let log_path = get_handler(&config_fields, LOG_PATH)?;
        let headers_path = get_handler(&config_fields, HEADERS_PATH)?;
        let blocks_path = get_handler(&config_fields, BLOCKS_PATH)?;
        let ipv6_enabled = parse_ipv6_enabled(&get_handler(&config_fields, IPV6_ENABLED)?)?;

        let mut dns = Vec::new();
        dns.extend(parse_dns_vector(&get_handler(&config_fields, DNS)?)?);

        let mut external_addresses = Vec::new();
        external_addresses.extend(parse_address_vector(&get_handler(&config_fields, EXTERNAL_ADDR)?)?);

        if dns.is_empty() && external_addresses.is_empty(){
            return Err(ConfigError::ErrorNoExternalAddressGiven);
        }

        Ok(Config {
            version,
            local_address,
            begin_time,
            log_path,
            headers_path,
            blocks_path,
            ipv6_enabled,
            dns,
            external_addresses,
        })
    }

    ///-
    fn _parse_date(data: &str) -> Result<u32, ConfigError>{
        let begin_time = parse_date(data)?;
                    
        if begin_time > (Utc::now().timestamp() as u32) {
            return Err(ConfigError::ErrorInvalidDate);
        }

        Ok(begin_time)
    }
}

/// A handler for opening the file containing the config's attributes, on error returns ErrorReadingFile
fn _open_config_handler(path: &str) -> Result<File, ConfigError> {
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(_) => Err(ConfigError::ErrorReadingFile),
    }
}

fn get_handler(config_fields: &HashMap<String, String>, field: &str) -> Result<String, ConfigError>{
    match config_fields.get(field){
        Some(data) => Ok(data.to_string()),
        None => {
            return Err(ConfigError::ErrorParameterNotFound);
        },
    }
}

/// It parses an string into a version number.
fn parse_version(data: &str) -> Result<i32, ConfigError>{
    data.parse::<i32>().map_err(|_| ConfigError::ErrorParsingVersion)
}

/// It parses an string into a vector of address.
fn parse_address_vector(addresses: &str) -> Result<Vec<([u8; 4], u16)>, ConfigError>{
    let splitted_addresses: Vec<&str> = addresses.split(ARRAY_DELIMETER).collect();

    let mut addresses = Vec::new();

    if splitted_addresses[0] != "" {
        for address in splitted_addresses{
            addresses.push(parse_address(address)?);
        }   
    }

    Ok(addresses)
}

/// It parses an string into a vector of dns.
fn parse_dns_vector(dns: &str) -> Result<Vec<(String, u16)>, ConfigError>{
    let splitted_dns: Vec<&str> = dns.split(ARRAY_DELIMETER).collect();
    
    let mut dns_vec = Vec::new();

    if splitted_dns[0] != "" {
        for dns in splitted_dns{
            dns_vec.push(parse_dns(dns)?);
        }   
    }

    Ok(dns_vec)
}

/// It parses an string into an address.
fn parse_address(address: &str) -> Result<([u8; 4], u16), ConfigError>{
    let splitted_address: Vec<&str> = address.split(PORT_DELIMETER).collect();
    let host = parse_ip_address(splitted_address[0])?;
    let port = parse_port(splitted_address[1])?;

    Ok((host, port))
}

/// It parses an string into a dns.
fn parse_dns(data: &str) -> Result<(String, u16), ConfigError>{
    let splitted_data: Vec<&str> = data.split(PORT_DELIMETER).collect();
    let port = parse_port(splitted_data[1])?;

    Ok((splitted_data[0].to_string(), port))
}

/// It parses an string into an IP.
fn parse_ip_address(address: &str) -> Result<[u8; 4], ConfigError> {
    let mut local_host: [u8; 4] = [0; 4];
    let splitter = address.split(IP_DELIMETER);

    for (i, number) in (0_usize..).zip(splitter) {

        local_host[i] = number.parse::<u8>().map_err(|_| ConfigError::ErrorParsingIP)?;
    }

    Ok(local_host)
}

/// It parses an string into a port.
fn parse_port(port: &str) -> Result<u16, ConfigError>{
    port.parse::<u16>().map_err(|_| ConfigError::ErrorParsingPort)
}

/// It receives a string representing a date and returns its timestamp at 00:00:00 in case of success, None otherwise.
fn parse_date(line: &str) -> Result<u32, ConfigError> {
    let complete_date = format!("{}T00:00:00Z", line);

    Ok(complete_date
        .parse::<DateTime<Utc>>()
        .map_err(|_| ConfigError::ErrorParsingDate)?
        .timestamp() as u32)
}

/// It parses an string into a boolean (enable IPV6).
fn parse_ipv6_enabled(data: &str) -> Result<bool, ConfigError>{
    data.parse::<bool>().map_err(|_| ConfigError::ErrorParsingIPV6Bool)
}


#[cfg(test)]
mod tests {
    use super::*;

    const STARTING_TIME: &str = "2023-04-10";
    const LOG_FILE_PATH: &str = "tests_txt/config_test_log.txt";
    const HEADERS_FILE_PATH: &str = "tests_txt/headers.bin";
    const BLOCKS_FILE_PATH: &str = "tests_txt/blocks.bin";


    // Auxiliar functions
    //=================================================================

    fn create_parameters(version: &str, local_address: &str, begin_time: &str, ipv6_enabled: bool, dns_vector: &str, ext_addr_vector: &str) -> HashMap<String, String>{
        let mut paramenters = HashMap::new();

        paramenters.insert(VERSION.to_string(), version.to_string());
        paramenters.insert(LOCAL_ADDRES.to_string(), local_address.to_string());
        paramenters.insert(BEGIN_TIME.to_string(), begin_time.to_string());
        paramenters.insert(LOG_PATH.to_string(), LOG_FILE_PATH.to_string());
        paramenters.insert(HEADERS_PATH.to_string(), HEADERS_FILE_PATH.to_string());
        paramenters.insert(BLOCKS_PATH.to_string(), BLOCKS_FILE_PATH.to_string());
        paramenters.insert(IPV6_ENABLED.to_string(), ipv6_enabled.to_string());
        paramenters.insert(DNS.to_string(), dns_vector.to_string());
        paramenters.insert(EXTERNAL_ADDR.to_string(), ext_addr_vector.to_string());

        paramenters
    }

    // Tests
    //=================================================================


    #[test]
    fn config_test_1_valid_file_creates_config() {
        assert!(Config::from_path("nodo.conf").is_ok());
    }

    #[test]
    fn config_test_2_invalid_file_cannot_create_config() {
        assert!(Config::from_path("invalid_file.conf").is_err());
    }

    #[test]
    fn config_test_3_saves_parameters_correctly() {
        let parameters = create_parameters(
            "70015",
            "127,0,0,1:1001",
            STARTING_TIME,
            false,
            "dns.first.example:18333;dns.second.example:18334",
            "127,0,0,2:18335;127,0,0,3:18333",
            );

        let expected_local_address = ([127, 0, 0, 1], 1001);
        let expected_begin_time_timestamp: u32 = 1681084800;
        let expected_dns = vec![("dns.first.example".to_string(), 18333), ("dns.second.example".to_string(), 18334)];
        let expected_external_addresses = vec![([127, 0, 0, 2], 18335), ([127, 0, 0, 3], 18333)];

        let config =
            Config::_from(parameters).expect("Could not create config from valid parameters.");

        assert_eq!(config.version, 70015);
        assert_eq!(config.local_address, expected_local_address);
        assert_eq!(config.begin_time, expected_begin_time_timestamp);
        assert_eq!(config.log_path, LOG_FILE_PATH.to_string());
        assert_eq!(config.headers_path, HEADERS_FILE_PATH.to_string());
        assert_eq!(config.blocks_path, BLOCKS_FILE_PATH.to_string());
        assert_eq!(config.ipv6_enabled, false);
        assert_eq!(config.dns, expected_dns);
        assert_eq!(config.external_addresses, expected_external_addresses);
    }

    #[test]
    fn config_test_4_invalid_amount_parameters_cannot_create_config() {
        let mut parameters = create_parameters(
            "70015",
            "127,0,0,1:1001",
            STARTING_TIME,
            false,
            "dns_vector:1",
            "1,2,3,4:2",
            );
        
        parameters.remove(LOG_PATH);

        assert!(Config::_from(parameters).is_err());
    }

    #[test]
    fn config_test_5_invalid_type_for_local_port_parameter_cannot_create_config() {
        let parameters = create_parameters(
            "70015",
            "127,0,0,1:this should be a u16",
            STARTING_TIME,
            true,
            "dns_vector:1",
            "1,2,3,4:2",
            );

        assert!(Config::_from(parameters).is_err());
    }

    #[test]
    fn config_test_6_future_begin_time_parameter_cannot_create_config() {
        let parameters = create_parameters(
            "70015",
            "127,0,0,1:1001",
            &Utc::now().date_naive().succ_opt().unwrap().to_string(),
            true,
            "dns_vector:1",
            "1,2,3,4:2",
            );

        assert!(Config::_from(parameters).is_err());
    }

    #[test]
    fn config_test_7_no_dns_parameter_can_create_config_correctly() {
        let parameters = create_parameters(
            "70015",
            "127,0,0,1:1001",
            STARTING_TIME,
            true,
            "",
            "1,2,3,4:1",
            );

        let expected_local_address = ([127, 0, 0, 1], 1001);
        let expected_begin_time_timestamp: u32 = 1681084800;
        let expected_dns = vec![];
        let expected_external_addresses = vec![([1, 2, 3, 4], 1)];


        let config =
        Config::_from(parameters).expect("Could not create config from valid parameters.");

        assert_eq!(config.version, 70015);
        assert_eq!(config.local_address, expected_local_address);
        assert_eq!(config.begin_time, expected_begin_time_timestamp);
        assert_eq!(config.log_path, LOG_FILE_PATH.to_string());
        assert_eq!(config.headers_path, HEADERS_FILE_PATH.to_string());
        assert_eq!(config.blocks_path, BLOCKS_FILE_PATH.to_string());
        assert_eq!(config.ipv6_enabled, true);
        assert_eq!(config.dns, expected_dns);
        assert_eq!(config.external_addresses, expected_external_addresses);  
    }

    #[test]
    fn config_test_8_no_ext_addr_parameter_can_create_config() {
        let parameters = create_parameters(
            "70015",
            "127,0,0,1:1001",
            STARTING_TIME,
            true,
            "dns_vector:1",
            "",
            );

        let expected_local_address = ([127, 0, 0, 1], 1001);
        let expected_begin_time_timestamp: u32 = 1681084800;
        let expected_dns = vec![("dns_vector".to_string(), 1)];
        let expected_external_addresses = vec![];


        let config =
        Config::_from(parameters).expect("Could not create config from valid parameters.");

        assert_eq!(config.version, 70015);
        assert_eq!(config.local_address, expected_local_address);
        assert_eq!(config.begin_time, expected_begin_time_timestamp);
        assert_eq!(config.log_path, LOG_FILE_PATH.to_string());
        assert_eq!(config.headers_path, HEADERS_FILE_PATH.to_string());
        assert_eq!(config.blocks_path, BLOCKS_FILE_PATH.to_string());
        assert_eq!(config.ipv6_enabled, true);
        assert_eq!(config.dns, expected_dns);
        assert_eq!(config.external_addresses, expected_external_addresses);  
    }

    #[test]
    fn config_test_9_no_dns_and_ext_addr_parameter_cannot_create_config() {
        let parameters = create_parameters(
            "70015",
            "127,0,0,1:1001",
            STARTING_TIME,
            true,
            "",
            "",
            );

        assert!(Config::_from(parameters).is_err());
    }
}
