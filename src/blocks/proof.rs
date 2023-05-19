use crate::blocks::blockchain::*;

/// Gets the target threshold of the n_bits specified, in order to 
fn get_target_threshold(n_bits: u32) -> [u8;32] {
    let n_bits_bytes = n_bits.to_le_bytes();
    let (exponent, significand) = n_bits_bytes.split_at(1);
    let exponent = exponent[0] as usize;

    let mut target_threshold = [0u8;32];
    let starting = (32 - exponent) as i32;
    for i in 0..3{
        if (starting + i >= 0) && (starting + i < 32){
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

    for i in 0..32{
        if block_header_hash[i] < threshold[i]{
            return true;
        }else if block_header_hash[i] > threshold[i]{
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::blocks::BlockHeader;
    use bitcoin_hashes::{sha256d, Hash};
    
    // Doing this test with a valid block header created by us is nearly impossible, because we can 
    // not create a valid block header without knowing the nonce, which is the value generated randomly.
    // The only test here checks if the proof of work is not valid

    #[test]
    fn proof_of_work_test_1_invalid_block_header() {
        let hash :[u8;32] = *sha256d::Hash::hash(b"test").as_byte_array();
        let merkle_hash :[u8;32] = *sha256d::Hash::hash(b"test merkle root").as_byte_array();
        let block_header = BlockHeader::new(70015, hash, merkle_hash);
        assert!(!validate_proof_of_work(&block_header));
    }
}