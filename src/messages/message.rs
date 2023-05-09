
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_tcp_stream::*;
    use std::net::Ipv4Addr;

    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001;

    fn version_message_without_user_agent_expected_bytes(timestamp: i64, rand: u64) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&timestamp.to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes());
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets());
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(0 as u8);
        //bytes_vector.extend_from_slice(&self.user_agent);
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }

    fn version_message_with_user_agent_expected_bytes() -> Vec<u8> {
        let rand: u64 = rand::thread_rng().gen();
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(&NODE_NETWORK.to_le_bytes());
        bytes_vector.extend_from_slice(&(Utc::now().timestamp() as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&(0 as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 2]);
        bytes_vector.extend_from_slice(&(8080 as u16).to_be_bytes());
        bytes_vector.extend_from_slice(&(NODE_NETWORK as u64).to_le_bytes());
        bytes_vector.extend_from_slice(&Ipv4Addr::from(LOCAL_HOST).to_ipv6_mapped().octets());
        bytes_vector.extend_from_slice(&LOCAL_PORT.to_be_bytes());
        bytes_vector.extend_from_slice(&rand.to_le_bytes());
        bytes_vector.push(253 as u8);
        bytes_vector.extend_from_slice(&(4 as u16).to_le_bytes());
        bytes_vector.extend_from_slice(&"test".as_bytes());
        bytes_vector.extend_from_slice(&(0 as i32).to_le_bytes());
        bytes_vector.push(0x01);
        bytes_vector
    }

    /* fn empty_header_message_expected_bytes() -> Vec<u8> {
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
    } */

    /* fn get_block_headers_message_expected_bytes() -> Vec <u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.push(1 as u8);
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector
    } */

    /* fn block_headers_message_expected_bytes() -> (Vec<u8>, BlockHeader, BlockHeader){
        let hash1 :[u8;32] = *sha256d::Hash::hash(b"test1").as_byte_array();
        let hash2 :[u8;32] = *sha256d::Hash::hash(b"test2").as_byte_array();
        let merkle_hash1 :[u8;32] = *sha256d::Hash::hash(b"test merkle root1").as_byte_array();
        let merkle_hash2 :[u8;32] = *sha256d::Hash::hash(b"test merkle root2").as_byte_array();
        
        let b_h1 = BlockHeader::new(70015, hash1, merkle_hash1); 
        let b_h2 = BlockHeader::new(70015, hash2, merkle_hash2);

        let mut expected_bytes = Vec::new();
        expected_bytes.push(2);
        expected_bytes.extend(b_h1.to_bytes());
        expected_bytes.extend(b_h2.to_bytes());
        (expected_bytes, b_h1, b_h2)
    } */
   
    // #[test]
    // fn test_to_bytes_1_version_message_without_user_agent() -> Result<(), MessageError> {
    //     let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
    //     let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
    //     let version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;

    //     let version_message_bytes = version_message.to_bytes();

    //     assert_eq!(
    //         version_message_bytes,
    //         version_message_without_user_agent_expected_bytes(
    //             version_message.timestamp,
    //             version_message.nonce
    //         )
    //     );
    //     Ok(())
    // }

    /* #[test]
    fn test_to_bytes_2_empty_header_message() -> Result<(), MessageError> {
        let header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;

        let header_message_bytes = header_message.to_bytes();

        assert_eq!(header_message_bytes, empty_header_message_expected_bytes());
        Ok(())
    } */

    /* #[test]

    fn test_to_bytes_3_non_empty_header_message() -> Result<(), MessageError> {
        let header_message = HeaderMessage::new("n_empty\0\0\0\0\0", &vec![1, 2, 3, 4])?;

        let header_message_bytes = header_message.to_bytes();

        assert_eq!(
            header_message_bytes,
            non_empty_header_message_expected_bytes()
        );
        Ok(())
    } */
    
    /* #[test]
    fn test_to_bytes_4_verack_message() -> Result<(), MessageError> {
        let verack_message = VerACKMessage::new()?;

        let verack_message_bytes = verack_message.to_bytes();

        assert_eq!(verack_message_bytes, Vec::new());
        Ok(())
    } */
    
    // #[test]
    // fn test_to_bytes_5_version_message_with_user_agent() -> Result<(), MessageError> {
    //     let mut expected_bytes = version_message_with_user_agent_expected_bytes();
    //     let version_message = VersionMessage::from_bytes(&mut expected_bytes.as_mut_slice())?;

    //     let version_message_bytes = version_message.to_bytes();

    //     assert_eq!(version_message_bytes, expected_bytes);
    //     Ok(())
    // }

    /* #[test]
    fn test_to_bytes_6_get_block_headers_message() -> Result<(), MessageError> {
        let expected_bytes = get_block_headers_message_expected_bytes();
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let mut vec_hash = Vec::new();
        vec_hash.push(hash);
        let get_block_headers_message = GetBlockHeadersMessage::new(70015, vec_hash,hash);

        let get_block_headers_message = get_block_headers_message.to_bytes();

        assert_eq!(get_block_headers_message, expected_bytes);
        Ok(())
    }  */

    /* #[test]
    fn test_to_bytes_8_block_headers_message() -> Result<(), MessageError> {
        
        let (expected_bytes, b_h1, b_h2) = block_headers_message_expected_bytes();
        let mut block_headers = Vec::new();
        block_headers.push(b_h1);
        block_headers.push(b_h2);

        let block_headers_message = BlockHeadersMessage::new(block_headers);

        assert_eq!(block_headers_message.to_bytes(), expected_bytes);
        Ok(())
    } */

    /* #[test]
    fn test_send_to_1_header_message() -> Result<(), MessageError> {
        let header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;
        let mut stream = MockTcpStream::new();

        header_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, header_message.to_bytes());
        Ok(())
    } */

    // #[test]
    // fn test_send_to_2_version_message() -> Result<(), MessageError> {
    //     let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
    //     let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
    //     let version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;
    //     let header_message = version_message.get_header_message()?;
    //     let mut stream = MockTcpStream::new();
    //     let mut expected_result = header_message.to_bytes();
    //     expected_result.extend(version_message.to_bytes());

    //     version_message.send_to(&mut stream)?;

    //     assert_eq!(stream.write_buffer, expected_result);
    //     Ok(())
    // }

    #[test]
    fn test_send_to_3_verack_message() -> Result<(), MessageError> {
        let ver_ack_message = VerACKMessage::new()?;
        let mut stream = MockTcpStream::new();

        ver_ack_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, empty_header_message_expected_bytes());
        Ok(())
    }

    /* #[test]
    fn test_send_to_4_get_block_headers_message()-> Result<(), MessageError> {
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let mut vec_hash = Vec::new();
        vec_hash.push(hash);
        let get_block_headers_message = GetBlockHeadersMessage::new(70015, vec_hash,hash);
        let mut stream = MockTcpStream::new();
        let header_message = get_block_headers_message.get_header_message()?;
        let mut expected_result = header_message.to_bytes();
        expected_result.extend(get_block_headers_message.to_bytes());
        get_block_headers_message.send_to(&mut stream)?;

        assert_eq!(stream.write_buffer, expected_result);
        Ok(())
    } */

    // #[test]
    // fn test_from_bytes_1_without_user_agent_version_message() -> Result<(), MessageError> {
    //     let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
    //     let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
    //     let expected_version_message = VersionMessage::new(70015, receiver_socket, sender_socket)?;

    //     let version_message =
    //         VersionMessage::from_bytes(&mut expected_version_message.to_bytes().as_mut_slice())?;

    //     assert_eq!(version_message, expected_version_message);
    //     Ok(())
    // }

    // #[test]
    // fn test_from_bytes_2_with_user_agent_version_message() -> Result<(), MessageError> {
    //     let receiver_socket = SocketAddr::from(([127, 0, 0, 2], 8080));
    //     let sender_socket = SocketAddr::from((LOCAL_HOST, LOCAL_PORT));
    //     let mut expected_version_message =
    //         VersionMessage::new(70015, receiver_socket, sender_socket)?;
    //     expected_version_message.user_agent_length = vec![253, 4, 0];
    //     expected_version_message.user_agent = Vec::from("test");

    //     let version_message =
    //         VersionMessage::from_bytes(&mut expected_version_message.to_bytes().as_mut_slice())?;

    //     assert_eq!(version_message, expected_version_message);
    //     Ok(())
    // }

    /* #[test]
    fn test_from_bytes_3_empty_header_message() -> Result<(), MessageError> {
        let expected_header_message = HeaderMessage::new("verack\0\0\0\0\0\0", &Vec::new())?;

        let header_message =
            HeaderMessage::from_bytes(&mut expected_header_message.to_bytes().as_mut_slice())?;

        assert_eq!(header_message, header_message);
        Ok(())
    } */

    /* #[test]
    fn test_from_bytes_4_non_empty_header_message() -> Result<(), MessageError> {
        let expected_header_message = HeaderMessage::new("version\0\0\0\0\0", &vec![1, 2, 3, 4])?;

        let header_message =
            HeaderMessage::from_bytes(&mut expected_header_message.to_bytes().as_mut_slice())?;

        assert_eq!(header_message, expected_header_message);
        Ok(())
    } */

    /* #[test]
    fn test_from_bytes_5_verack_message_from_empty_slice() -> Result<(), MessageError> {
        let expected_verack_message = VerACKMessage::new()?;

        let verack_message =
            VerACKMessage::from_bytes(&mut expected_verack_message.to_bytes().as_mut_slice())?;

        assert_eq!(verack_message, expected_verack_message);
        Ok(())
    } */

    /* #[test]
    fn test_from_bytes_6_verack_message_from_non_empty_slice() -> Result<(), MessageError> {
        let expected_verack_message = VerACKMessage::new()?;
        let mut expected_bytes = expected_verack_message.to_bytes();
        expected_bytes.extend(vec![1, 2, 3, 4]);

        let verack_message =
            VerACKMessage::from_bytes(&mut expected_bytes.as_mut_slice()).unwrap_err();

        assert_eq!(verack_message, MessageError::ErrorCreatingVerAckMessage);
        Ok(())
    } */

    /* #[test]
    fn test_from_bytes_7_get_block_headers_message() -> Result<(), MessageError> {
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let mut vec_hash = Vec::new();
        vec_hash.push(hash);
        let expected_get_block_headers_message = GetBlockHeadersMessage::new(70015, vec_hash,hash);

        let  get_block_headers_message=
        GetBlockHeadersMessage::from_bytes(&mut expected_get_block_headers_message.to_bytes().as_mut_slice())?;

        assert_eq!(get_block_headers_message, expected_get_block_headers_message);
        Ok(())
    } */

    /* #[test]
    fn test_from_bytes_8_block_headers_message() -> Result<(), MessageError> {
        let (mut expected_bytes, b_h1, b_h2) = block_headers_message_expected_bytes();
        let mut block_headers = Vec::new();
        block_headers.push(b_h1);
        block_headers.push(b_h2);

        let expected_block_headers_message = BlockHeadersMessage::new(block_headers);

        let block_headers_message = BlockHeadersMessage::from_bytes(&mut expected_bytes)?;
        assert_eq!(block_headers_message, expected_block_headers_message);
        Ok(())

    } */
}
*/