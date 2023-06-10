use crate::{
    blocks::transaction::*,
    utils::{btc_errors::BlockChainError, variable_length_integer::VarLenInt},
};

use bitcoin_hashes::{sha256d, Hash};
use chrono::Utc;
use rand::prelude::*;

const BLOCKHEADER_SIZE: usize = 80;
const MINIMAL_BLOCK_SIZE: usize = 81;

/// Struct that represents the header of a block
#[derive(Debug, PartialEq, Clone)]
pub struct BlockHeader {
    version: i32,
    prev_hash: [u8; 32],
    merkle_root_hash: [u8; 32],
    pub time: u32,
    n_bits: u32,
    nonce: u32,
}

/// Struct that represents a block
#[derive(Debug)]
pub struct Block {
    header: BlockHeader,
    transaction_count: VarLenInt,
    transactions: Vec<Transaction>,
}

impl BlockHeader {
    /// It creates and returns a BlockHeader with the values passed as parameters.
    /// The time is set to the current time and the nonce is set to a random number.
    pub fn new(
        version: i32,
        prev_hash: [u8; 32],
        merkle_root_hash: [u8; 32],
        n_bits: u32,
    ) -> BlockHeader {
        BlockHeader {
            version,
            prev_hash,
            merkle_root_hash,
            time: Utc::now().timestamp() as u32,
            n_bits,
            nonce: rand::thread_rng().gen(),
        }
    }

    /// It creates and returns a slice of bytes with the values of the BlockHeader.
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

    /// Wrapper for _from_bytes
    pub fn from_bytes(slice: &[u8]) -> Result<BlockHeader, BlockChainError> {
        if slice.len() != BLOCKHEADER_SIZE {
            return Err(BlockChainError::ErrorCreatingBlockHeader);
        }

        match Self::_from_bytes(slice) {
            Some(block_header) => Ok(block_header),
            None => Err(BlockChainError::ErrorCreatingBlockHeader),
        }
    }

    /// It creates and returns a BlockHeader from a slice of bytes.
    fn _from_bytes(slice: &[u8]) -> Option<BlockHeader> {
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

    /// It returns the hash of the BlockHeader.
    pub fn hash(&self) -> [u8; 32] {
        *sha256d::Hash::hash(&self.to_bytes()).as_byte_array()
    }

    /// It returns the n_bits of the BlockHeader. The n_bits are used to calculate the target threshold.
    pub fn get_n_bits(&self) -> u32 {
        self.n_bits
    }

    /// It returns the merkle root hash of the BlockHeader.
    pub fn get_merkle_root(&self) -> &[u8; 32] {
        &self.merkle_root_hash
    }
}

impl Block {
    /// It creates and returns a Block with the values passed as parameters.
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Block {
        let transaction_count = VarLenInt::new(transactions.len());
        Block {
            header,
            transaction_count,
            transactions,
        }
    }

    /// It returns the Block as a slice of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = self.header.to_bytes();
        bytes_vector.extend_from_slice(&self.transaction_count.to_bytes());
        for transaction in &self.transactions {
            bytes_vector.extend_from_slice(&transaction.to_bytes());
        }
        bytes_vector
    }

    /// Wrapper for _from_bytes
    pub fn from_bytes(slice: &[u8]) -> Result<Block, BlockChainError> {
        if slice.len() < MINIMAL_BLOCK_SIZE {
            return Err(BlockChainError::ErrorCreatingBlock);
        }

        match Self::_from_bytes(slice) {
            Some(version_message) => Ok(version_message),
            None => Err(BlockChainError::ErrorCreatingBlock),
        }
    }

    /// It creates and returns a Block from a slice of bytes.
    fn _from_bytes(slice: &[u8]) -> Option<Block> {
        let (header_bytes, mut slice) = slice.split_at(BLOCKHEADER_SIZE);
        let header = BlockHeader::from_bytes(header_bytes).ok()?;
        let transaction_count = VarLenInt::from_bytes(slice);
        (_, slice) =
            slice.split_at(transaction_count.amount_of_bytes());

        let mut transactions = Vec::new();

        let mut i = 0;
        while i < transaction_count.to_usize() {
            let transaction = Transaction::from_bytes(&slice).ok()?;
            (_, slice) = slice.split_at(transaction.amount_of_bytes());
            i += 1;
            transactions.push(transaction);
        }

        Some(Block {
            header,
            transaction_count,
            transactions,
        })
    }

    pub fn amount_of_bytes(&self) -> usize {
        self.to_bytes().len()
    }

    /// It returns the time when the Block was created.
    pub fn time(&self) -> u32 {
        self.header.time
    }

    /// It returns the header of the block.
    pub fn get_header(&self) -> BlockHeader {
        BlockHeader { 
            version: self.header.version, 
            prev_hash: self.header.prev_hash, 
            merkle_root_hash: self.header.merkle_root_hash, 
            time: self.header.time, 
            n_bits: self.header.n_bits, 
            nonce: self.header.nonce }
    }

    /// It returns the transactions of the block.
    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn header_hash(&self) -> [u8;32]{
        self.get_header().hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_hashes::{sha256d, Hash};

    // Auxiliar functions
    //=================================================================

    fn block_header_expected_bytes() -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&(70015 as i32).to_le_bytes());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test").as_byte_array());
        bytes_vector.extend_from_slice(sha256d::Hash::hash(b"test merkle root").as_byte_array());
        bytes_vector.extend_from_slice(&(0 as u32).to_le_bytes());
        bytes_vector.extend_from_slice(&(0x30c31b18 as u32).to_be_bytes());
        bytes_vector.extend_from_slice(&(14082023 as u32).to_le_bytes());
        bytes_vector
    }

    fn block_expected_bytes() -> Vec<u8> {
        let mut expected_bytes = block_header_expected_bytes();
        expected_bytes.push(2);
        //temporal hasta que definiamos que son las transacciones
        let t1 = Transaction::new(70015, Vec::new(), Vec::new(), 123);
        let t2 = Transaction::new(70015, Vec::new(), Vec::new(), 123);
        expected_bytes.extend(t1.to_bytes());
        expected_bytes.extend(t2.to_bytes());
        expected_bytes
    }

    // Tests
    //=================================================================

    #[test]
    fn test_blockheader_1_to_bytes() {
        let block_header = BlockHeader {
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
    fn test_blockheader_2_from_bytes() {
        let mut block_header_bytes = block_header_expected_bytes();
        let block_header = BlockHeader::from_bytes(&mut block_header_bytes).unwrap();

        assert_eq!(block_header.to_bytes(), block_header_bytes);
    }

    #[test]
    fn test_block_1_to_bytes() {
        let header = BlockHeader {
            version: 70015,
            prev_hash: *sha256d::Hash::hash(b"test").as_byte_array(),
            merkle_root_hash: *sha256d::Hash::hash(b"test merkle root").as_byte_array(),
            time: 0,
            n_bits: 0x30c31b18,
            nonce: 14082023,
        };
        let transaction_count = VarLenInt::new(2);
        let t1 = Transaction::new(70015, Vec::new(), Vec::new(), 123);
        let t2 = Transaction::new(70015, Vec::new(), Vec::new(), 123);

        let block = Block {
            header,
            transaction_count,
            transactions: vec![t1, t2],
        };

        assert_eq!(block_expected_bytes(), block.to_bytes());
    }

    #[test]
    fn test_block2_from_bytes() {
        let mut expected_block_bytes = block_expected_bytes();
        let block_header = Block::from_bytes(&mut expected_block_bytes).unwrap();

        assert_eq!(block_header.to_bytes(), expected_block_bytes);
    }
}
