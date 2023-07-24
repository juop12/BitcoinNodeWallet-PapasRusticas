use crate::blocks::blockchain::*;
use bitcoin_hashes::{sha256d, Hash};

/// Gets the target threshold of the n_bits specified
fn get_target_threshold(n_bits: u32) -> [u8; 32] {
    let n_bits_bytes = n_bits.to_le_bytes();
    let (exponent, significand) = n_bits_bytes.split_at(1);
    let exponent = exponent[0] as usize;

    let mut target_threshold = [0u8; 32];
    let starting = (32 - exponent) as i32;
    for i in 0..3 {
        if (starting + i >= 0) && (starting + i < 32) {
            target_threshold[(starting + i) as usize] = significand[i as usize];
        }
    }

    target_threshold
}

/// Validates the proof of work of a block, by checking if the hash of the block header is less than the target threshold
pub fn validate_proof_of_work(block_header: &BlockHeader) -> bool {
    let n_bits = block_header.get_n_bits();
    let mut block_header_hash = block_header.hash();
    let threshold = get_target_threshold(n_bits);
    block_header_hash.reverse();

    block_header_hash <= threshold
}

/// Returns a hash of the concatenation of two hashes.
pub fn hash_pairs_for_merkle_tree(hash_1: [u8; 32], hash_2: [u8; 32]) -> [u8; 32] {
    let mut total_hash: Vec<u8> = Vec::from(hash_1);
    total_hash.extend(hash_2);

    let new_hash = sha256d::Hash::hash(total_hash.as_slice());
    new_hash.to_byte_array()
}

/// Calculates the merkle root of the header of a block.
/// Receives the hashes of its transactions, and returns the merkle root.
fn calculate_merkle_tree_level(
    mut hash_vector: Vec<[u8; 32]>,
    merkle_tree: &mut Vec<Vec<[u8; 32]>>,
) {
    if hash_vector.len() == 1 {
        merkle_tree.push(hash_vector);
        return;
    }
    if hash_vector.len() % 2 != 0 {
        let last_transaction_hash = hash_vector[hash_vector.len() - 1];
        hash_vector.push(last_transaction_hash);
    }
    let mut new_hash_vector = Vec::new();
    for i in 0..hash_vector.len() / 2 {
        let new_hash = hash_pairs_for_merkle_tree(hash_vector[2 * i], hash_vector[2 * i + 1]);
        new_hash_vector.push(new_hash);
    }
    calculate_merkle_tree_level(new_hash_vector, merkle_tree);
    merkle_tree.push(hash_vector);
}

/// Validates the proof of inclusion of a block, by checking if the merkle root of the block
/// header is equal to the merkle root calculated from its transactions. Returns true if it is valid.
pub fn validate_block_proof_of_inclusion(block: &Block) -> bool {
    let hash_vector = block.get_tx_hashes();
    if hash_vector.is_empty() {
        return true;
    }

    let mut merkle_tree = Vec::new();
    calculate_merkle_tree_level(hash_vector, &mut merkle_tree);
    let header_merkle_root = *block.get_header().get_merkle_root();
    merkle_tree[0][0] == header_merkle_root
}

pub struct HashPair {
    pub left: [u8; 32],
    pub right: [u8; 32],
    path_comes_from: Side,
}

enum Side {
    Left,
    Right,
}

impl HashPair {
    fn new(left: [u8; 32], right: [u8; 32], side: Side) -> HashPair {
        HashPair {
            left,
            right,
            path_comes_from: side,
        }
    }

    /// Returns true if the hash is equal to the correct side of the HashPair
    pub fn equals_path_side(&self, hash: [u8; 32]) -> bool {
        match self.path_comes_from {
            Side::Left => self.left == hash,
            Side::Right => self.right == hash,
        }
    }

    /// Hashes the result of concatenating, left and right
    pub fn hash(&self) -> [u8; 32] {
        hash_pairs_for_merkle_tree(self.left, self.right)
    }
}

/// Returns a vector of hashpairs that can be used to veify with the also returned merkle
pub fn proof_of_transaction_included_in(
    transaction_hash: [u8; 32],
    block: &Block,
) -> (Vec<HashPair>, [u8; 32]) {
    let hash_vector = block.get_tx_hashes();
    if hash_vector.is_empty() {
        return (Vec::new(), block.get_header().merkle_root_hash);
    }

    let mut merkle_tree = Vec::new();
    calculate_merkle_tree_level(hash_vector, &mut merkle_tree);

    let mut level_position: usize = 0;

    for (i, tx_hash) in merkle_tree[merkle_tree.len() - 1].iter().enumerate() {
        if *tx_hash == transaction_hash {
            level_position = i;
            break;
        }
    }

    let mut merkle_proof = Vec::new();

    for current_level in merkle_tree.iter().rev() {
        if current_level.len() == 1 {
            break;
        }

        let hash_pair = if (level_position % 2) == 0 {
            HashPair::new(
                current_level[level_position],
                current_level[level_position + 1],
                Side::Left,
            )
        } else {
            HashPair::new(
                current_level[level_position - 1],
                current_level[level_position],
                Side::Right,
            )
        };
        merkle_proof.push(hash_pair);

        level_position /= 2;
    }

    (merkle_proof, merkle_tree[0][0])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::blocks::{transaction::Transaction, BlockHeader};
    use bitcoin_hashes::{sha256d, Hash};

    const VALID_HEADER_BYTES: [u8; 80] = [
        0, 128, 154, 33, 97, 0, 155, 57, 119, 6, 109, 83, 36, 160, 202, 81, 110, 211, 12, 33, 242,
        251, 163, 225, 189, 198, 99, 91, 39, 0, 0, 0, 0, 0, 0, 0, 81, 36, 107, 173, 77, 174, 133,
        197, 186, 33, 40, 129, 186, 247, 243, 121, 96, 34, 123, 34, 217, 248, 194, 216, 2, 183, 11,
        96, 57, 6, 158, 34, 104, 145, 103, 100, 140, 202, 39, 25, 74, 168, 232, 213,
    ];

    // Auxiliar functions
    //=================================================================

    fn get_transactions(lock_time: u32) -> Transaction {
        Transaction::new(70015, Vec::new(), Vec::new(), lock_time)
    }

    fn get_merkle_root(transactions: &Vec<Transaction>) -> [u8; 32] {
        let mut first_joined_hashes = Vec::from(transactions[0].hash());
        first_joined_hashes.extend(transactions[1].hash());
        let first_hash = sha256d::Hash::hash(&first_joined_hashes.as_slice());

        let mut second_joined_hashes = Vec::from(transactions[2].hash());
        second_joined_hashes.extend(transactions[2].hash());
        let second_hash = sha256d::Hash::hash(&second_joined_hashes.as_slice());

        let mut final_joined_hashes = Vec::from(first_hash.to_byte_array());
        final_joined_hashes.extend(second_hash.to_byte_array());
        sha256d::Hash::hash(&&final_joined_hashes.as_slice()).to_byte_array()
    }

    fn get_block(valid: bool) -> Block {
        let mut transactions = Vec::new();
        for i in 0..3 {
            transactions.push(get_transactions(i));
        }
        let header: BlockHeader;
        if valid {
            header = BlockHeader::new(70015, [0u8; 32], get_merkle_root(&transactions), 0);
        } else {
            header = BlockHeader::new(70015, [0u8; 32], [0u8; 32], 0);
        }
        Block::new(header, transactions)
    }

    // Tests
    //=================================================================

    #[test]
    fn proof_of_work_test_1_invalid_block_header() {
        let hash: [u8; 32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let merkle_hash: [u8; 32] = *sha256d::Hash::hash(b"test merkle root").as_byte_array();
        let block_header = BlockHeader::new(70015, hash, merkle_hash, 0);
        assert!(!validate_proof_of_work(&block_header));
    }

    #[test]
    fn proof_of_work_test_2_valid_block_header() {
        let header = BlockHeader::from_bytes(&VALID_HEADER_BYTES).unwrap();
        assert!(validate_proof_of_work(&header))
    }

    #[test]
    fn proof_of_inclusion_test_1_invalidad_merkle_root() {
        let block = get_block(false);
        assert!(!validate_block_proof_of_inclusion(&block))
    }

    #[test]
    fn proof_of_inclusion_test_1_validad_merkle_root() {
        let block = get_block(true);
        assert!(validate_block_proof_of_inclusion(&block))
    }
}
