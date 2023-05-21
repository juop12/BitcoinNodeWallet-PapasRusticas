use super::utils::*;


const VERACK_MSG_NAME: &str = "verack\0\0\0\0\0\0";


/// Message used to acknoledge 2 nodes have sent Version Messages.
#[derive(Debug, PartialEq)]
pub struct VerACKMessage {}

impl Message for VerACKMessage {
    type MessageType = VerACKMessage;
    /// Implements the trait send_to for VerACKMessage, sends a VerACKMessage trough the tcp_stream,
    /// returns an error if the message could not be sent.
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), MessageError> {
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)
    }
    /// Returns an empty vector of bytes, since the VerACKMessage has no payload.
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Returns a VerACKMessage if the slice of bytes is empty, otherwise returns a MessageError.
    fn from_bytes(slice: &mut [u8]) -> Result<Self::MessageType, MessageError> {
        if !slice.is_empty() {
            return Err(MessageError::ErrorCreatingVerAckMessage);
        }
        Ok(VerACKMessage {})
    }

    /// Returns a copy of the header message
    fn get_header_message(&self) -> Result<HeaderMessage, MessageError> {
        HeaderMessage::new(VERACK_MSG_NAME, &self.to_bytes())
    }
}

impl VerACKMessage {
    /// Constructor for the struct VerACKMessage, returns an instance of a VerACKMessage
    pub fn new() -> Result<VerACKMessage, MessageError> {
        Ok(VerACKMessage {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mock_tcp_stream::MockTcpStream;


    const START_STRING_TEST_NET: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];


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

    // Tests
    //=================================================================

    #[test]
    fn verack_message_test_1_send_to() -> Result<(), MessageError> {
        let verack_msg = VerACKMessage::new()?;
        let mut stream = MockTcpStream::new();

        verack_msg.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, empty_header_message_expected_bytes());
        Ok(())
    }

    #[test]
    fn verack_message_test_2_to_bytes() -> Result<(), MessageError> {
        let verack_msg= VerACKMessage::new()?;

        let verack_msg_bytes = verack_msg.to_bytes();

        assert_eq!(verack_msg_bytes, Vec::new());
        Ok(())
    }

    #[test]
    fn verack_message_test_3_from_bytes_from_empty_slice() -> Result<(), MessageError> {
        let expected_verack_msg = VerACKMessage::new()?;

        let verack_msg =
            VerACKMessage::from_bytes(&mut expected_verack_msg.to_bytes().as_mut_slice())?;

        assert_eq!(verack_msg, expected_verack_msg);
        Ok(())
    }

    #[test]
    fn verack_message_test_4_from_bytes_from_non_empty_slice() -> Result<(), MessageError> {
        let expected_verack_msg = VerACKMessage::new()?;
        let mut expected_bytes = expected_verack_msg.to_bytes();
        expected_bytes.extend(vec![1, 2, 3, 4]);

        let verack_msg =
            VerACKMessage::from_bytes(&mut expected_bytes.as_mut_slice()).unwrap_err();

        assert_eq!(verack_msg, MessageError::ErrorCreatingVerAckMessage);
        Ok(())
    }

}
