use chrono::Utc;

const BLOCKHEADER_SIZE: usize = 80; 

pub enum BlockChainError {
    ErrorCreatingBlock,
    ErrorSendingBlock,
    ErrorCreatingBlockHeader,
    ErrorSendingBlockHeader,
}

#[derive(Debug)]
pub struct BlockHeader {
    version: i32,
    prev_hash: [u8; 32],
    merkle_root_hash: [u8; 32],
    time: u32,
    nBits: u32,
    nonce: u32,
}

#[derive(Debug)]
pub struct Block {
    header: BlockHeader,
    //transaction_count: usize, // 0 for now.
    //transactions: Vec<Transaction>,
}

impl BlockHeader{

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        bytes_vector.extend_from_slice(&self.prev_hash);
        bytes_vector.extend_from_slice(&self.merkle_root_hash);
        bytes_vector.extend_from_slice(&self.time.to_le_bytes());
        bytes_vector.extend_from_slice(&self.nBits.to_be_bytes());
        bytes_vector.extend_from_slice(&self.nonce.to_le_bytes());
        bytes_vector
    }

    fn from_bytes(slice: &mut [u8]) -> Result<Self::BlockHeader, BlockChainError> {
        if slice.len() != BLOCKHEADER_SIZE {
            return Err(BlockChainError::ErrorCreatingBlockHeader);
        }

        match Self::_from_bytes(slice) {
            Some(block_header) => Ok(block_header),
            None => Err(BlockChainError::ErrorCreatingBlockHeader),
        }
    }

    fn _from_bytes(slice: &mut [u8]) -> Option<BlockHeader> {

        let version = i32::from_le_bytes(slice[0..4].try_into().ok()?);
        let prev_hash = slice[4..36].try_into().ok()?;
        let merkle_root_hash = slice[36..68].try_into().ok()?;
        let time = u32::from_le_bytes(slice[68..72].try_into().ok()?);
        let nBits = u32::from_le_bytes(slice[72..76].try_into().ok()?);
        let nonce = u32::from_le_bytes(slice[76..80].try_into().ok()?);

        Some(BlockHeader {
            version,
            prev_hash,
            merkle_root_hash,
            time,
            nBits,
            nonce,
        })
    }
}

impl Block{
    /*
    fn send_to<T: Read + Write>(&self, receiver_stream: &mut T) -> Result<(), BlockChainError> {
        let header_message = self.get_header_message()?;
        header_message.send_to(receiver_stream)?;

        match receiver_stream.write(self.to_bytes().as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(BlockChainError::ErrorSendingBlock),
        }
    }
    */

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.header.to_bytes());
        //bytes_vector.extend_from_slice(&self.transaction_count.to_le_bytes());
        //bytes_vector.extend_from_slice(&self.transaction.to_le_bytes());
        bytes_vector
    }

    fn from_bytes(slice: &mut [u8]) -> Result<Self::Block, BlockChainError> {
        //if slice.len() < MINIMAL_VERSION_MESSAGE_SIZE {
        //    return Err(MessageError::ErrorCreatingVersionMessage);
        //}

        match Self::_from_bytes(slice) {
            Some(version_message) => Ok(version_message),
            None => Err(Err(BlockChainError::ErrorCreatingBlock)),
        }
    }

    fn _from_bytes(slice: &mut [u8]) -> Option<HeaderMessage> {

        let header = BlockHeader::from_bytes(slice[0..80].try_into().ok()?);

        //let transaction_count = slice[80..??].try_into().ok()?;
        //let transactions = slice[??..].try_into().ok()?;
        Some(Block {
            header: BlockHeader,
            //transaction_count: usize, // 0 for now.
            //transactions: Vec<Transaction>,
        })
    }    
}