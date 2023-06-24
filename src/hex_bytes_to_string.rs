pub fn get_string_representation_from_bytes(bytes_vec: &mut Vec<u8>) -> String {
    bytes_vec.reverse();
    get_hex_from_bytes(bytes_vec)
}

pub fn get_hex_from_bytes(bytes_vec: &Vec<u8>) -> String{
    bytes_vec
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<String>()
}
