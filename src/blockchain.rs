use chrono::Utc;
use rand::prelude::*;
use bitcoin_hashes::{sha256d, Hash};

const BLOCKHEADER_SIZE: usize = 80; 

#[derive(Debug)]
pub enum BlockChainError {
    ErrorCreatingBlock,
    ErrorSendingBlock,
    ErrorCreatingBlockHeader,
    ErrorSendingBlockHeader,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockHeader {
    version: i32,
    pub prev_hash: [u8; 32],
    merkle_root_hash: [u8; 32],
    time: u32,
    n_bits: u32,
    nonce: u32,
}

#[derive(Debug)]
pub struct Block {
    header: BlockHeader,
    //transaction_count: usize, // 0 for now.
    //transactions: Vec<Transaction>,
}

impl BlockHeader{

    pub fn new(version: i32,prev_hash: [u8; 32],merkle_root_hash: [u8; 32]) -> BlockHeader {
        BlockHeader{
            version,
            prev_hash,
            merkle_root_hash,
            time: Utc::now().timestamp() as u32,
            n_bits: 0,
            nonce: rand::thread_rng().gen(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        bytes_vector.extend_from_slice(&self.prev_hash);
        bytes_vector.extend_from_slice(&self.merkle_root_hash);
        bytes_vector.extend_from_slice(&self.time.to_le_bytes());
        bytes_vector.extend_from_slice(&self.n_bits.to_be_bytes());
        bytes_vector.extend_from_slice(&self.nonce.to_le_bytes());
        bytes_vector
    }

    pub fn from_bytes(slice: &mut [u8]) -> Result<BlockHeader, BlockChainError> {
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
        let n_bits = u32::from_be_bytes(slice[72..76].try_into().ok()?);
        let nonce = u32::from_le_bytes(slice[76..80].try_into().ok()?);

        Some(BlockHeader {
            version,
            prev_hash,
            merkle_root_hash,
            time,
            n_bits,
            nonce,
        })
    }

    pub fn hash(&self) -> [u8;32]{
        *sha256d::Hash::hash(&self.to_bytes()).as_byte_array()
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.header.to_bytes());
        //bytes_vector.extend_from_slice(&self.transaction_count.to_le_bytes());
        //bytes_vector.extend_from_slice(&self.transaction.to_le_bytes());
        bytes_vector
    }

    fn from_bytes(&self, slice: &mut [u8]) -> Result<Block, BlockChainError> {
        if slice.len() < BLOCKHEADER_SIZE {
            return Err(BlockChainError::ErrorCreatingBlock);
        }

        match Self::_from_bytes(slice) {
            Some(version_message) => Ok(version_message),
            None => Err(BlockChainError::ErrorCreatingBlock),
        }
    }

    fn _from_bytes(slice: &mut [u8]) -> Option<Block> {

        let header = match BlockHeader::from_bytes(&mut slice[..BLOCKHEADER_SIZE]){
            Ok(header) => header,
            Err(_) => return None,
        };

        //let transaction_count = slice[80..??].try_into().ok()?;
        //let transactions = slice[??..].try_into().ok()?;
        Some(Block {
            header,
            //transaction_count: usize, // 0 for now.
            //transactions: Vec<Transaction>,
        })
    } 
}

#[cfg(test)]
mod tests {
    use super::*;

    use bitcoin_hashes::{sha256d, Hash};
    
    fn block_header_expected_bytes() -> Vec<u8>{
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test merkle root").as_byte_array());
        bytes_vector.extend_from_slice(&(0 as u32).to_le_bytes());
        bytes_vector.extend_from_slice(&(0x30c31b18 as u32).to_be_bytes());
        bytes_vector.extend_from_slice(&(14082023 as u32).to_le_bytes());
        bytes_vector
    }


    #[test]
    fn test_blockheader_1_to_bytes(){
        let block_header = BlockHeader{
            version: 70015, 
            prev_hash: *sha256d::Hash::hash(b"test").as_byte_array(),
            merkle_root_hash: *sha256d::Hash::hash(b"test merkle root").as_byte_array(),
            time: 0,
            n_bits: 0x30c31b18,
            nonce: 14082023,
        };

        assert_eq!(block_header_expected_bytes(), block_header.to_bytes());
    }

    #[test]
    fn test_blockheader_2_from_bytes(){
        let mut block_header_bytes = block_header_expected_bytes();
        let block_header = BlockHeader::from_bytes(&mut block_header_bytes).unwrap();

        assert_eq!(block_header.to_bytes(), block_header_bytes);

    }
}