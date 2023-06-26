/// This function takes a vector of bytes and returns a string representation of the bytes
/// in hexadecimal format.
pub fn get_string_representation_from_bytes(bytes_vec: &mut Vec<u8>) -> String {
    bytes_vec.reverse();
    get_hex_from_bytes(bytes_vec)
}

/// This function takes a hexadecimal string and returns a vector of bytes.
pub fn get_hex_from_bytes(bytes_vec: &Vec<u8>) -> String{
    bytes_vec
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<String>()
}
