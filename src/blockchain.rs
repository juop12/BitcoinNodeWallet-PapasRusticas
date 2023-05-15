use chrono::Utc;
use rand::prelude::*;
use bitcoin_hashes::{sha256d, Hash};
use crate::messages::block_message;
use crate::messages::utils::calculate_variable_length_integer;

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
    pub header: BlockHeader,
    pub transaction_count: Vec<u8>, // 0 for now.
    //transactions: Vec<Transaction>,
    pub transactions: Vec<u8>,
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

    pub fn time(&self) -> u32{
        self.time.clone()
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
        let mut bytes_vector = self.header.to_bytes();
        bytes_vector.extend_from_slice(&self.transaction_count);
        bytes_vector.extend_from_slice(&self.transactions);
        bytes_vector
    }

    pub fn from_bytes(slice: &mut [u8]) -> Result<Block, BlockChainError> {
        if slice.len() < BLOCKHEADER_SIZE {
            return Err(BlockChainError::ErrorCreatingBlock);
        }

        match Self::_from_bytes(slice) {
            Some(version_message) => Ok(version_message),
            None => Err(BlockChainError::ErrorCreatingBlock),
        }
    }

    fn _from_bytes(slice: &mut [u8]) -> Option<Block> {

        let (header_bytes, slice) = slice.split_at_mut(BLOCKHEADER_SIZE);
        let header = BlockHeader::from_bytes(header_bytes).ok()?;

        let (transaction_count, count_amount_of_bytes, amount_of_transactions) = calculate_variable_length_integer(slice);
        let (_count_bytes ,transactions_bytes) = slice.split_at_mut(count_amount_of_bytes);
        Some(Block {
            header,
            transaction_count, // 0 for now.
            transactions: Vec::from(transactions_bytes),
        })
    } 

    pub fn time(&self) -> u32{
        self.header.time()
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

    fn block_expected_bytes()->Vec<u8>{
        let mut expected_bytes =  block_header_expected_bytes();
        expected_bytes.push(2);
        //temporal hasta que definiamos que son las transacciones
        for _ in 0..100{
            expected_bytes.push(0)
        }
        expected_bytes
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

    
    #[test]
    fn test_block_1_to_bytes(){
        let header = BlockHeader{
            version: 70015, 
            prev_hash: *sha256d::Hash::hash(b"test").as_byte_array(),
            merkle_root_hash: *sha256d::Hash::hash(b"test merkle root").as_byte_array(),
            time: 0,
            n_bits: 0x30c31b18,
            nonce: 14082023,
        };
        let transaction_count:Vec<u8> = vec![2];
        let mut transactions:Vec<u8> = Vec::new();
        for _ in 0..100{
            transactions.push(0)
        }

        let block =Block{
            header,
            transaction_count,
            transactions,
        };

        assert_eq!(block_expected_bytes(), block.to_bytes());
    }

    #[test]
    fn test_block2_from_bytes(){
        let mut expected_block_bytes = block_expected_bytes();
        let block_header = Block::from_bytes(&mut expected_block_bytes).unwrap();

        assert_eq!(block_header.to_bytes(), expected_block_bytes);
    }
}