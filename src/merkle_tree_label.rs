use node::blocks::proof::{HashPair, hash_pairs_for_merkle_tree};
use gtk::Label;

static LEFT_BRANCH: &str = " /";
const RIGHT_BRANCH: &str = "\\ ";

fn build_matrix(hash_pairs: &Vec<HashPair>) -> Vec<Vec<String>>{
    let height = 2 * (hash_pairs.len()) + 1;
    let wide = 2 * (hash_pairs.len()) + 4;
    let mut column: Vec<String> = Vec::with_capacity(wide);
    for _ in 0..wide {
        column.push("  ".to_string());
    }
    let mut matrix: Vec<Vec<String>> = Vec::with_capacity(height);
    for _ in 0..height {
        matrix.push(column.clone());
    }
    matrix
}

pub fn funcion_re_fachera(hash_pairs: &mut Vec<HashPair>) -> String {
    hash_pairs.reverse();
    let mut matrix = build_matrix(hash_pairs);
    let left_hash_pos = hash_pairs.len() + 2;
    let right_hash_pos = hash_pairs.len() + 4;
    let mut hashes_positions = vec![left_hash_pos, right_hash_pos];
    matrix[0][hash_pairs.len() + 2] = String::from("MR");
    let merkle_root = hash_pairs_for_merkle_tree(hash_pairs[0].left, hash_pairs[0].right);
    let mut left_hash: [u8; 32] = merkle_root;
    let mut right_hash: [u8; 32] = merkle_root;
    let mut current_row = 1;
    let mut current_pair = 1;
    for pair in hash_pairs {
        let left_hash_str = format!("{:?}L", current_pair);
        let right_hash_str = format!("{:?}R", current_pair);
        let hashed_pair = hash_pairs_for_merkle_tree(pair.left, pair.right);
        println!("Hashed pair: {:?}\n", hashed_pair);
        println!("left pair: {:?}\n", left_hash);
        println!("right pair: {:?}\n", right_hash);
        if hashed_pair == left_hash {
            hashes_positions[0] = hashes_positions[0] - 1;
            hashes_positions[1] = hashes_positions[1] - 1;
        } else {
            hashes_positions[0] = hashes_positions[0] + 1;
            hashes_positions[1] = hashes_positions[1] + 1;
        }
        matrix[current_row][hashes_positions[0]] = LEFT_BRANCH.to_string();
        matrix[current_row][hashes_positions[1]] = RIGHT_BRANCH.to_string();
        current_row += 1;
        matrix[current_row][hashes_positions[0]] = left_hash_str;
        matrix[current_row][hashes_positions[1]] = right_hash_str;
        left_hash = pair.left;
        right_hash = pair.right;
        current_row += 1;
        current_pair += 1;
    }
    let rows_concatenated: Vec<String> = matrix.iter()
    .map(|row| row.join(""))
    .collect();

    // Unir las filas con el carácter de nueva línea
    let result: String = rows_concatenated.join("\n");  
    result
}