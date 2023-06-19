
use std::{fmt::Write, num::ParseIntError};

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

#[cfg(test)]
mod tests {
    use bitcoin_hashes::{Hash, sha256d};
    use secp256k1::{Message, PublicKey, ecdsa::Signature, Secp256k1};
    use super::decode_hex;

    #[test]
    fn tx_signature_test() {        
        let tx_unsigned = decode_hex("0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000001976a914a802fc56c704ce87c42d7c92eb75e7896bdc41ae88acfeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac1943060001000000").unwrap();
        let pk_sec = decode_hex("0349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278a").unwrap();
        let sig_der = decode_hex("3045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed").unwrap();
        
        let valid = validate_signature(tx_unsigned, pk_sec, sig_der);
        
        assert_eq!(Ok(()), valid);
    }

    #[test]
    fn tx_papas_al_horno_test() {
        let tx_unsigned = decode_hex("0100000001596977cdb54902ea45c82f0ee9dbff76cf1cac84e37680513e3196c1fe5c51c7010000001976a914d8e3142858a1f5a8340888f3d4e0fd3c330ee27d88acffffffff0240420f00000000001976a9141e5e45669c7b2293534fa554141bcd2c5d113ee388aca7c00400000000001976a914d8e3142858a1f5a8340888f3d4e0fd3c330ee27d88ac0000000001000000").unwrap();
        let pk_sec = decode_hex("0357dd612f9e51d01c5cac8435cb6c401571507cafe309e4a9bb48a40b118bf8ff").unwrap();
        let sig_der = decode_hex("3045022100f1515fac5411ed2b5f318733d32a7f9b5eeeed109d9eabe843b3d6ff85d3bf7802206157dc3134432c8324a5e66cc38af08cbecd6a418dbef30e018b3ef3eb5b2c35").unwrap();
        
        let valid = validate_signature(tx_unsigned, pk_sec, sig_der);

        assert_eq!(Ok(()), valid);
    }

    fn validate_signature(tx_unsigned: Vec<u8>, pksec: Vec<u8>, sig_der: Vec<u8>) -> Result<(), secp256k1::Error> {
        let secp = Secp256k1::new();

        let z = sha256d::Hash::hash(&tx_unsigned);
    
        let pk = PublicKey::from_slice(&pksec).unwrap();

        let sig = Signature::from_der(&sig_der).unwrap();

        let msg = Message::from_slice(&z.to_byte_array()).unwrap();

        return secp.verify_ecdsa(&msg, &sig, &pk);
    }


}