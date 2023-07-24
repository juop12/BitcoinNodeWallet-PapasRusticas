use node::blocks::proof::{hash_pairs_for_merkle_tree, HashPair};

/// Constants representing left and right branches in the diagram, and the reporesentation of the merkle root
static LEFT_BRANCH: &str = " /";
const RIGHT_BRANCH: &str = "\\ ";
const MERKLE_ROOT_REPRESENTATION: &str = "MR";

/// This function initializes and returns an empty matrix of strings
fn build_matrix(height: usize, width: usize) -> Vec<Vec<String>> {
    let empty = "  ".to_string();
    vec![vec![empty; width]; height]
}

/// This function generates a Merkle proof of inclusion in text format
pub fn draw_merkle_proof_of_inclusion_tree(hash_pairs: &mut Vec<HashPair>) -> String {
    hash_pairs.reverse();

    let height = hash_pairs.len() * 2 + 1;
    let width = hash_pairs.len() * 2 + 4;
    let mut matrix = build_matrix(height, width);

    let mut hashes_positions: Vec<i32> =
        vec![(hash_pairs.len() + 2) as i32, (hash_pairs.len() + 4) as i32];

    if hash_pairs.is_empty() {
        return String::from(MERKLE_ROOT_REPRESENTATION);
    }
    matrix[0][(hash_pairs.len() + 2)] = String::from(MERKLE_ROOT_REPRESENTATION);
    let mut left_hash = hash_pairs_for_merkle_tree(hash_pairs[0].left, hash_pairs[0].right);

    for (i, pair) in hash_pairs.iter().enumerate() {
        let left_hash_str = format!("{:?}L", i + 1);
        let right_hash_str = format!("{:?}R", i + 1);
        let hashed_pair = hash_pairs_for_merkle_tree(pair.left, pair.right);

        let shift_direction: i32 = if hashed_pair == left_hash { -1 } else { 1 };
        hashes_positions = hashes_positions
            .iter()
            .map(|&x| x + shift_direction)
            .collect();

        let current_row = i * 2 + 1; // Calculate the current row index based on the current pair index

        matrix[current_row][hashes_positions[0] as usize] = LEFT_BRANCH.to_string();
        matrix[current_row][hashes_positions[1] as usize] = RIGHT_BRANCH.to_string();

        matrix[current_row + 1][hashes_positions[0] as usize] = left_hash_str;
        matrix[current_row + 1][hashes_positions[1] as usize] = right_hash_str;

        left_hash = pair.left;
    }

    // Join all the rows with spaces, then join all of those strings together with line breaks
    matrix
        .iter()
        .map(|row| row.join(" "))
        .collect::<Vec<String>>()
        .join("\n")
}
