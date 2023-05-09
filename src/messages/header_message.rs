use super::utils::*;
use bitcoin_hashes::{sha256d, Hash};


const MESAGE_HEADER_SIZE: usize = 24;
const START_STRING_TEST_NET: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];
const COMMAND_NAME_ERROR: &str = "\0\0\0\0\0\0\0\0\0\0\0\0";
const COMMAND_NAME_SIZE: usize = 12;


/// Struct that represents a header message in the bitcoin protocol
#[derive(Debug, PartialEq, Clone)]
pub struct HeaderMessage {
    start_string: [u8; 4],
    command_name: [u8; COMMAND_NAME_SIZE],
    payload_size: u32,
    checksum: [u8; 4],
}

impl Message for HeaderMessage {
    type MessageType = HeaderMessage;

    /// Sends a header message trough the tcp_stream
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError> {
        match receiver_stream.write(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MessageError::ErrorSendingHeaderMessage),
        }
    }

    /// Returns an array of bytes with the header message in the format specified in the bitcoin protocol
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.start_string);
        bytes_vector.extend_from_slice(&self.command_name);
        bytes_vector.extend_from_slice(&self.payload_size.to_le_bytes());
        bytes_vector.extend_from_slice(&self.checksum);
        bytes_vector
    }

    /// Receives a slice of bytes and returns a HeaderMessage if everything went Ok or a
    /// MessageError if the conversion from bytes to HeaderMessage failed.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError> {
        if slice.len() != MESAGE_HEADER_SIZE {
            return Err(MessageError::ErrorCreatingHeaderMessage);
        }
        match Self::_from_bytes(slice) {
            Some(header_msg) => Ok(header_msg),
            None => Err(MessageError::ErrorCreatingHeaderMessage),
        }
    }

    //Returns a copy of the header message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        Ok(self.clone())
    }
}

impl HeaderMessage {
    /// Receives a command name and a payload size and returns an instance of a HeaderMessage with
    /// all its necesary attributes initialized, according to the p2p bitcoin protocol
    pub fn new(command_name: &str, payload: &Vec<u8>) -> Result<HeaderMessage, MessageError> {
        if command_name.len() > COMMAND_NAME_SIZE {
            return Err(MessageError::ErrorCreatingHeaderMessage);
        }

        let mut command_bytes = Vec::from(command_name.as_bytes());
        while command_bytes.len() < COMMAND_NAME_SIZE {
            command_bytes.push(0);
        }
        
        let mut command_bytes_fixed_size = [0u8; COMMAND_NAME_SIZE];
        command_bytes_fixed_size.copy_from_slice(command_bytes.as_slice());
        //let payload_size = size_of_val(payload.as_slice()) as u32;

        let payload_size: u32 = payload.len() as u32;

        let hash = sha256d::Hash::hash(payload.as_slice());
        let hash_value = hash.as_byte_array();
        let checksum: [u8; 4] = match hash_value[..4].try_into() {
            Ok(array) => array,
            Err(_) => return Err(MessageError::ErrorCreatingHeaderMessage),
        };

        let header_msg = HeaderMessage {
            start_string: START_STRING_TEST_NET,
            command_name: command_bytes_fixed_size,
            payload_size,
            checksum, //(SHA256(SHA256(<empty string>)))
        };
        Ok(header_msg)
    }

    /// Returns the payload size of the header message
    pub fn get_payload_size(&self) -> u32 {
        self.payload_size
    }

    /// Returns the command name of the header message
    pub fn get_command_name(&self) -> String {
        match String::from_utf8(Vec::from(self.command_name)) {
            Ok(string) => string,
            Err(_) => String::from(COMMAND_NAME_ERROR),
        }
    }

    /// Receives a slice of bytes and returns an Option<HeaderMessage>, initialices the fields of
    /// the HeaderMessage with the values in the slice, if any step in the middle of the conversion
    /// fails, returns None.
    fn _from_bytes(slice: &mut [u8]) -> Option<HeaderMessage> {
        let start_string = slice[0..4].try_into().ok()?;
        let command_name = slice[4..16].try_into().ok()?;
        let payload_size = u32::from_le_bytes(slice[16..20].try_into().ok()?);
        let checksum = slice[20..24].try_into().ok()?;

        Some(HeaderMessage {
            start_string,
            command_name,
            payload_size,
            checksum,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_tcp_stream::MockTcpStream;


    // Auxiliar functions
    //=================================================================

    fn empty_header_message_expected_bytes() -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&START_STRING_TEST_NET);
        bytes_vector.extend_from_slice(&"verack\0\0\0\0\0\0".as_bytes());
        bytes_vector.extend_from_slice(&(0 as u32).to_le_bytes());
        bytes_vector.extend_from_slice([0x5d, 0xf6, 0xe0, 0xe2].as_slice());
        bytes_vector
    }

    fn non_empty_header_message_expected_bytes() -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&START_STRING_TEST_NET);
        bytes_vector.extend_from_slice(&"n_empty\0\0\0\0\0".as_bytes());
        bytes_vector.extend_from_slice(&(4 as u32).to_le_bytes());
        bytes_vector.extend_from_slice([0x8d, 0xe4, 0x72, 0xe2].as_slice());
        bytes_vector
    }

    // Tests
    //=================================================================

    #[test]
    fn header_message_test_1_to_bytes_empty_header_message() -> Result<(), MessageError> {
        let hm = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;

        let hm_bytes = hm.to_bytes();

        assert_eq!(hm_bytes, empty_header_message_expected_bytes());
        Ok(())
    }

    #[test]
    fn header_message_test_2_to_bytes_non_empty_header_message() -> Result<(), MessageError> {
        let hm = HeaderMessage::new("n_empty\0\0\0\0\0", &vec![1, 2, 3, 4])?;

        let hm_bytes = hm.to_bytes();

        assert_eq!(
            hm_bytes,
            non_empty_header_message_expected_bytes()
        );
        Ok(())
    }

    #[test]
    fn header_message_test_3_send_to() -> Result<(), MessageError> {
        let hm = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;
        let mut stream = MockTcpStream::new();

        hm.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, hm.to_bytes());
        Ok(())
    }

    #[test]
    fn header_message_test_4_from_bytes_empty_header_message() -> Result<(), MessageError> {
        let expected_hm = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;

        let hm =
            HeaderMessage::from_bytes(&mut expected_hm.to_bytes().as_mut_slice())?;

        assert_eq!(hm, expected_hm);
        Ok(())
    }

    #[test]
    fn header_message_test_5_from_bytes_non_empty_header_message() -> Result<(), MessageError> {
        let expected_hm = HeaderMessage::new("version\0\0\0\0\0", &vec![1, 2, 3, 4])?;

        let hm =
            HeaderMessage::from_bytes(&mut expected_hm.to_bytes().as_mut_slice())?;

        assert_eq!(hm, expected_hm);
        Ok(())
    }
}