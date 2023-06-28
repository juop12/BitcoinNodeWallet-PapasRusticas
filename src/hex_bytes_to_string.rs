/// This function takes a vector of bytes and returns a string representation of the bytes
/// in hexadecimal format. It is reversed because the bytes are stored in little endian format
/// and we want to display them in big endian format.
pub fn get_string_representation_from_bytes(bytes_vec: &mut [u8]) -> String {
    bytes_vec.reverse();
    get_hex_from_bytes(bytes_vec)
}

/// This function takes a hexadecimal string and returns a vector of bytes.
pub fn get_hex_from_bytes(bytes_vec: &[u8]) -> String {
    bytes_vec
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<String>()
}
