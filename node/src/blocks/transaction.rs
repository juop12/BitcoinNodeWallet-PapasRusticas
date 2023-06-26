use crate::utils::{btc_errors::TransactionError, variable_length_integer::VarLenInt};
use bitcoin_hashes::{sha256d, hash160, Hash};
use secp256k1::{SecretKey, Message, PublicKey, constants::PUBLIC_KEY_SIZE};


const MIN_BYTES_TX_IN: usize = 41;
const MIN_BYTES_TX_OUT: usize = 9;
const MIN_BYTES_TRANSACTION: usize = 10;
const OUTPOINT_BYTES: usize = 36;

const P2PKH_SCRIPT_LENGTH: usize = 25; 
const OP_DUP: u8 = 0x76;
const OP_DUP_POSITION: usize = 0;
const OP_HASH160: u8 = 0xA9;
const OP_HASH160_POSITION: usize = 1;
const P2PKH_HASH_LENGTH:u8 = 0x14;
const P2PKH_HASH_LENGTH_POSITION:usize = 2;
const OP_EQUALVERIFY:u8 = 0x88;
const OP_EQUALVERIFY_POSITION: usize = 23;
const OP_CHECKSIG: u8 = 0xAC;
const OP_CHECKSIG_POSITION: usize = 24;

const SIGHASH_ALL :[u8;4] = [0x01, 0x00, 0x00, 0x00]; // Already in BigEndian

/// Struct that represents the Outpoint, that is used in the TxIn struct.

#[derive(Eq, Hash, Debug, PartialEq, Clone, Copy)]
pub struct Outpoint {
    pub hash: [u8; 32],
    pub index: u32,
}

/// Struct that represents the TxIn used in the struct Transaction.
#[derive(Debug, PartialEq)]
pub struct TxIn {
    pub previous_output: Outpoint,
    script_length: VarLenInt,
    signature_script: Vec<u8>,
    sequence: u32, // u32::MAX; Se usa el maximo u32.
}

/// Struct that represents the TxOut used in the struct Transaction
#[derive(Debug, PartialEq)]
pub struct TxOut {
    pub value: i64,
    pub pk_script_length: VarLenInt,
    pub pk_script: Vec<u8>,
}

/// Implementation of a Transaction in a Bitcoin block.
#[derive(Debug, PartialEq)]
pub struct Transaction {
    version: i32,
    tx_in_count: VarLenInt,
    pub tx_in: Vec<TxIn>,
    tx_out_count: VarLenInt,
    pub tx_out: Vec<TxOut>,
    lock_time: u32, // siempre va 0.
}

impl Outpoint {
    ///Creates a new Outpoint
    pub fn new(hash: [u8; 32], index: u32) -> Outpoint {
        Outpoint { hash, index }
    }

    ///Returns the contents of Outpoint as a bytes vecotr
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::from(self.hash);
        bytes.extend(self.index.to_le_bytes());
        bytes
    }

    ///If the bytes given can form a valid Outpoint, it creates it, if not returns error
    pub fn from_bytes(slice: &[u8]) -> Result<Outpoint, TransactionError> {
        if slice.len() != OUTPOINT_BYTES {
            return Err(TransactionError::ErrorCreatingOutpointFromBytes);
        }
        let (hash_bytes, index_bytes) = slice.split_at(32);
        let hash: [u8; 32] = match hash_bytes.try_into() {
            Ok(hash) => hash,
            Err(_) => return Err(TransactionError::ErrorCreatingOutpointFromBytes),
        };
        let index: u32 = match index_bytes.try_into() {
            Ok(index) => u32::from_le_bytes(index),
            Err(_) => return Err(TransactionError::ErrorCreatingOutpointFromBytes),
        };

        Ok(Outpoint::new(hash, index))
    }
}

impl TxOut {
    /// Creates a new TxOut.
    pub fn new(value: i64, pk_script: Vec<u8>) -> TxOut {
        let pk_script_length = VarLenInt::new(pk_script.len());
        TxOut {
            value,
            pk_script_length,
            pk_script,
        }
    }

    /// Returns the contents of Outpoint as a bytes vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.value.to_le_bytes());
        bytes_vector.extend_from_slice(&self.pk_script_length.to_bytes());
        bytes_vector.extend_from_slice(&self.pk_script);
        bytes_vector
    }

    /// If the bytes given can form a valid TxOut, it creates it, if not returns error.
    /// It is important to note that not all bytes passed will be used. The function
    /// will only use the bytes needed to create the TxOut
    pub fn from_bytes(slice: &[u8]) -> Result<TxOut, TransactionError> {
        if slice.len() < MIN_BYTES_TX_OUT {
            return Err(TransactionError::ErrorCreatingTxOutFromBytes);
        }

        let value = match slice[0..8].try_into().ok() {
            Some(value) => i64::from_le_bytes(value),
            None => return Err(TransactionError::ErrorCreatingTxOutFromBytes),
        };
        match VarLenInt::from_bytes(&slice[8..]){
            Some(pk_script_length) => {
                let (_left_bytes, slice) = slice.split_at(8 + pk_script_length.amount_of_bytes());
                let pk_script = slice[0..pk_script_length.to_usize()].to_vec();

                Ok(TxOut {
                    value,
                    pk_script_length,
                    pk_script,
                })
            },
            None => Err(TransactionError::ErrorCreatingTxOutFromBytes),
        }
        
    }

    /// Returns the amount of bytes that the TxOut need to represent the TxOut.
    fn amount_of_bytes(&self) -> usize {
        self.to_bytes().len()
    }

    /// Returns true if the pk_script of the tx_out follows the p2pkh protocol
    pub fn pk_hash_under_p2pkh_protocol(&self) -> Option<&[u8]>{
        if self.pk_script_length.to_usize() != P2PKH_SCRIPT_LENGTH{
            return None;
        }
        if self.pk_script[OP_DUP_POSITION] != OP_DUP{
            return None;
        }
        if self.pk_script[OP_HASH160_POSITION] != OP_HASH160{
            return None;
        }
        if self.pk_script[P2PKH_HASH_LENGTH_POSITION] != P2PKH_HASH_LENGTH{
            return None;
        }
        if self.pk_script[OP_EQUALVERIFY_POSITION] != OP_EQUALVERIFY{
            return None;
        }
        if self.pk_script[OP_CHECKSIG_POSITION] != OP_CHECKSIG{
            return None;
        }
        return Some(&self.pk_script[3..23])
    }

    ///-
    pub fn clone(&self) -> TxOut{
        TxOut { 
            value: self.value,
            pk_script_length: VarLenInt::new(self.pk_script_length.to_usize()), 
            pk_script: self.pk_script.clone() 
        }
    }

    ///-
    pub fn belongs_to(&self, pk_hash: [u8;20]) -> bool{
        if let Some(owner_pk_hash) = self.pk_hash_under_p2pkh_protocol(){
            return pk_hash == owner_pk_hash;
        }
        false
    }
}

impl TxIn {
    /// Creates a new TxIn
    pub fn new(previous_output: Outpoint, signature_script: Vec<u8>, sequence: u32) -> TxIn {
        let script_length = VarLenInt::new(signature_script.len());
        TxIn {
            previous_output,
            script_length,
            signature_script,
            sequence,
        }
    }

    ///-
    pub fn create_unsigned_with(previous_output: Outpoint) -> TxIn{
        TxIn::new(previous_output, Vec::new(), u32::MAX)
    }
    
    ///-
    pub fn insert_script_signature(&mut self, signature_script: Vec<u8>){
        self.script_length = VarLenInt::new(signature_script.len());
        self.signature_script = signature_script;
    }

    /// Returns the contents of TxIn as a bytes vector
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.previous_output.to_bytes());
        bytes_vector.extend_from_slice(&self.script_length.to_bytes());
        bytes_vector.extend_from_slice(&self.signature_script);
        bytes_vector.extend_from_slice(&self.sequence.to_le_bytes());
        bytes_vector
    }

    /// If the bytes given can form a valid TxIn, it creates it, if not returns error.
    /// It is important to note that not all bytes passed will be used. The function
    /// will only use the bytes needed to create the TxIn
    fn from_bytes(slice: &[u8]) -> Result<TxIn, TransactionError> {
        if slice.len() < MIN_BYTES_TX_IN {
            return Err(TransactionError::ErrorCreatingTxInFromBytes);
        }
        let (prev_out_bytes, slice) = slice.split_at(OUTPOINT_BYTES);
        let previous_output = Outpoint::from_bytes(prev_out_bytes)?;
        let script_length = match VarLenInt::from_bytes(slice){
            Some(script_length) => script_length,
            None => return Err(TransactionError::ErrorCreatingTxInFromBytes),
        };
        let (_script_length_bytes, slice) = slice.split_at(script_length.amount_of_bytes());
        if slice.len() < script_length.to_usize() + 4 {
            return Err(TransactionError::ErrorCreatingTxInFromBytes);
        }
        let (signature_script_bytes, slice) = slice.split_at(script_length.to_usize());
        let signature_script = signature_script_bytes.to_vec();
        let (sequence_bytes, _slice) = slice.split_at(4);
        let sequence = match sequence_bytes.try_into() {
            Ok(sequence_bytes) => u32::from_le_bytes(sequence_bytes),
            Err(_) => return Err(TransactionError::ErrorCreatingTxInFromBytes),
        };

        Ok(TxIn {
            previous_output,
            script_length,
            signature_script,
            sequence,
        })
    }

    /// Returns the amount of bytes needed to represent the TxIn.
    fn amount_of_bytes(&self) -> usize {
        self.to_bytes().len()
    }

    /// Returns true if the pk_script of the tx_out follows the p2pkh protocol
    pub fn belongs_to(&self, pub_key: &PublicKey) -> bool{
        let sig_len = match VarLenInt::from_bytes(&self.signature_script){
            Some(sig_len) => sig_len,
            None => return false,
        };
        
        if sig_len.to_usize() >= self.script_length.to_usize(){
            return false;
        }

        let (_sig, bytes_left) = self.signature_script.split_at(sig_len.to_usize() + sig_len.amount_of_bytes());
        
        let pub_key_len = match VarLenInt::from_bytes(&bytes_left){
            Some(pub_key_len) => pub_key_len,
            None => return false,
        };

        if pub_key_len.to_usize() != PUBLIC_KEY_SIZE{
            return false
        }

        let total_len = sig_len.amount_of_bytes() + sig_len.to_usize() + pub_key_len.amount_of_bytes() + pub_key_len.to_usize();

        if total_len != self.script_length.to_usize(){
            return false
        }
        
        match PublicKey::from_slice(&bytes_left[sig_len.amount_of_bytes()..]){
            Ok(script_pub_key) => script_pub_key == *pub_key,
            Err(_) => false,
        }
    }
}

impl Transaction {
    /// Creates a new Transaction
    pub fn new(
        version: i32,
        tx_in_vector: Vec<TxIn>,
        tx_out_vector: Vec<TxOut>,
        lock_time: u32,
    ) -> Transaction {
        Transaction {
            version,
            tx_in_count: VarLenInt::new(tx_in_vector.len()),
            tx_in: tx_in_vector,
            tx_out_count: VarLenInt::new(tx_out_vector.len()),
            tx_out: tx_out_vector,
            lock_time,
        }
    }

    ///-
    pub fn create(amount: i64, fee: i64, unspent_outpoints: Vec<Outpoint>, unspent_balance: i64, pub_key: PublicKey, priv_key: SecretKey, address: [u8;25]) -> Result<Transaction, TransactionError>{
        
        let change: i64 = unspent_balance - amount - fee;
        let tx_out_vector = create_tx_out_vector(change, amount, pub_key, address);
        
        let tx_in_vector = create_unsigned_tx_in_vector(unspent_outpoints);
        
        let mut raw_tx = Transaction::new(1, tx_in_vector, tx_out_vector,0);

        let mut signature_vec: Vec<Vec<u8>> = Vec::new();

        for i in 0..raw_tx.tx_in_count.to_usize(){
            raw_tx.tx_in[i].insert_script_signature(Vec::from(get_pk_script_from_pubkey(pub_key)));
            
            let signature_script = raw_tx.get_signature_script(pub_key, priv_key)?;
            signature_vec.push(signature_script);

            raw_tx.tx_in[i].insert_script_signature(Vec::new());
        }
        
        for (signature_script, tx_in) in signature_vec.into_iter().zip(raw_tx.tx_in.iter_mut()){
            tx_in.insert_script_signature(signature_script);
        }

        Ok(raw_tx)
    }
    
    //firmar
    //  tenemos la raw transaction
    //  1- metemos en el campo sig_script el pk_script (si habia algo se saca, para chequear)
    //  2- metemos el hash_type al final de la raw tx, probablemente SIGHASH_ALL(01000000)
    //  3-  z = int::from_big_endian  hash256(modified_transaccion.to_bytes)
    //  4- der = private_key.sign(z).der()
    //  5- sig = der + SIGHASH_ALL.to_bytes(1, 'big')  The signature is actually a combination of the DER signature and the hash type, (suma) which is SIGHASH_ALL in our case
    
    //  6- sec = private_key.point.sec() // ES LA PUBKEY COMPRESSED DE 33 BYTES!!!
    //  7- sig_script = [varlenInt(sig), sig, Varlenint(sec), sec]
    ///-
    fn get_signature_script(&self, pub_key: PublicKey, priv_key: SecretKey)-> Result<Vec<u8>, TransactionError>{
        
        // 1
        let mut tx_bytes = self.to_bytes();
        
        // 2
        tx_bytes.extend(SIGHASH_ALL);
        
        // 3 Conseguimos z.
        let message = Message::from_hashed_data::<sha256d::Hash>(&tx_bytes);

        // 4
        //let secret_key = SecretKey::from_slice(&priv_key).map_err(|_| TransactionError::ErrorCreatingSignature)?;
        let signature = priv_key.sign_ecdsa(message);
        
        // 5
        let mut signature = signature.serialize_der().to_vec();
        signature.push(SIGHASH_ALL[0]);

        // 6 Ya tenemos el SEC, es la pub_key compressed.

        // 7
        let signature_script = assemble_signature_script(signature, pub_key);

        Ok(signature_script)
    }

    /// Returns the contents of Transaction as a bytes vector
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        bytes_vector.extend_from_slice(&self.tx_in_count.to_bytes());
        for tx in &self.tx_in {
            bytes_vector.extend_from_slice(&tx.to_bytes());
        }
        bytes_vector.extend_from_slice(&self.tx_out_count.to_bytes());
        for tx in &self.tx_out {
            bytes_vector.extend_from_slice(&tx.to_bytes());
        }
        bytes_vector.extend_from_slice(&self.lock_time.to_le_bytes());
        bytes_vector
    }

    /// If the bytes given can form a valid Transaction, it creates it, if not returns error.
    pub fn from_bytes(slice: &[u8]) -> Result<Transaction, TransactionError> {
        if slice.len() < MIN_BYTES_TRANSACTION {
            return Err(TransactionError::ErrorCreatingTxInFromBytes);
        }
        match Self::_from_bytes(slice) {
            Some(transaction) => Ok(transaction),
            None => Err(TransactionError::ErrorCreatingTxInFromBytes),
        }
    }

    /// It receives the slice of bytes and checks if it can form a valid Transaction by converting the bytes into
    /// the corresponding fields. If it can, it returns the Transaction, if not, it returns None
    fn _from_bytes(slice: &[u8]) -> Option<Transaction> {
        let version = i32::from_le_bytes(slice[0..4].try_into().ok()?);
        let tx_in_count = match VarLenInt::from_bytes(&slice[4..]){
            Some(var_len_int) => var_len_int,
            None => return None,
        };
        let (mut _used_bytes, mut slice) = slice.split_at(4 + tx_in_count.amount_of_bytes());

        let mut tx_in: Vec<TxIn> = Vec::new();
        for _ in 0..tx_in_count.to_usize() {
            let tx = TxIn::from_bytes(slice).ok()?;
            (_used_bytes, slice) = slice.split_at(tx.amount_of_bytes());
            tx_in.push(tx);
        }

        let tx_out_count = match VarLenInt::from_bytes(slice){
            Some(var_len_int) => var_len_int,
            None => return None
        };
        
        (_used_bytes, slice) = slice.split_at(tx_out_count.amount_of_bytes());

        let mut tx_out: Vec<TxOut> = Vec::new();
        for _ in 0..tx_out_count.to_usize() {
            let tx = TxOut::from_bytes(slice).ok()?;
            (_used_bytes, slice) = slice.split_at(tx.amount_of_bytes());
            tx_out.push(tx);
        }

        let (lock_time_bytes, _slice) = slice.split_at(4);
        let lock_time = u32::from_le_bytes(lock_time_bytes.try_into().ok()?);

        Some(Transaction {
            version,
            tx_in_count,
            tx_in,
            tx_out_count,
            tx_out,
            lock_time,
        })
    }

    /// Returns the amount of bytes needed to represent the Transaction.
    pub fn amount_of_bytes(&self) -> usize {
        self.to_bytes().len()
    }

    ///-
    pub fn hash(&self) -> [u8; 32] {
        *sha256d::Hash::hash(&self.to_bytes()).as_byte_array()
    }

    pub fn get_ballance_regarding(&self, ){

    }
}

///-
fn create_unsigned_tx_in_vector(unspent_outpoints: Vec<Outpoint>) -> Vec<TxIn>{
    let mut tx_in_vector = Vec::new();

    for outpoint in unspent_outpoints{
        let txin = TxIn::create_unsigned_with(outpoint);
        tx_in_vector.push(txin);
    }

    tx_in_vector
}

///-
fn assemble_signature_script(signature: Vec<u8> ,pub_key: PublicKey) -> Vec<u8>{
    let len_sig = VarLenInt::new(signature.len());
    let len_sec = VarLenInt::new(pub_key.serialize().len());

    let mut signature_script = Vec::from(len_sig.to_bytes());        
    signature_script.extend(signature);
    signature_script.extend(len_sec.to_bytes());
    signature_script.extend(pub_key.serialize());

    signature_script
}

///-
fn create_tx_out_vector(change: i64, amount: i64, pub_key: PublicKey, address: [u8;25]) -> Vec<TxOut>{
    let mut receiver_pk_hash: [u8;20] = [0; 20];
    receiver_pk_hash.copy_from_slice(&address[1..21]);
    let pk_script_receiver = get_pk_script(receiver_pk_hash);
    let tx_out_receiver = TxOut::new(amount, Vec::from(pk_script_receiver));

    let pk_script_change = get_pk_script_from_pubkey(pub_key);
    let tx_out_change = TxOut::new(change, Vec::from(pk_script_change));      

    vec![tx_out_receiver, tx_out_change]
} 

///-
pub fn get_pk_script_from_pubkey(pub_key: PublicKey) -> [u8; P2PKH_SCRIPT_LENGTH]{
    let pk_hash = hash160::Hash::hash(&pub_key.serialize());
    
    get_pk_script(pk_hash.to_byte_array())
}


///-
fn get_pk_script(pk_hash: [u8; 20]) -> [u8; P2PKH_SCRIPT_LENGTH]{
    let mut pk_script: [u8; P2PKH_SCRIPT_LENGTH] = [0; P2PKH_SCRIPT_LENGTH];

    pk_script[OP_DUP_POSITION] = OP_DUP;
    pk_script[OP_HASH160_POSITION] = OP_HASH160;
    pk_script[P2PKH_HASH_LENGTH_POSITION] = P2PKH_HASH_LENGTH;
    pk_script[3..23].copy_from_slice(&pk_hash);
    pk_script[OP_EQUALVERIFY_POSITION] = OP_EQUALVERIFY;
    pk_script[OP_CHECKSIG_POSITION] = OP_CHECKSIG; 
    
    pk_script
}

#[cfg(test)]
mod tests {
    use super::*;

    // Auxiliar functions
    //=================================================================

    fn outpoint_32_byte_array() -> [u8; 32] {
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]
    }

    fn tx_in_expected_bytes() -> Vec<u8> {
        let mut bytes_vector: Vec<u8> = Vec::new();
        let outpoint = Outpoint::new(outpoint_32_byte_array(), 4);
        bytes_vector.extend_from_slice(&outpoint.to_bytes());
        bytes_vector.extend_from_slice(&VarLenInt::new(5).to_bytes());
        bytes_vector.extend_from_slice(&[1, 2, 3, 4, 5]);
        bytes_vector.extend_from_slice(&(123 as u32).to_le_bytes());
        bytes_vector
    }

    fn tx_out_expected_bytes() -> Vec<u8> {
        let mut bytes_vector: Vec<u8> = Vec::new();
        bytes_vector.extend((5 as i64).to_le_bytes());
        bytes_vector.extend(VarLenInt::new(5).to_bytes());
        bytes_vector.extend([1, 2, 3, 4, 5]);
        bytes_vector
    }

    fn transaction_expected_bytes_with_tx_in_and_tx_out() -> (Vec<u8>, Vec<TxIn>, Vec<TxOut>) {
        let mut bytes_vector: Vec<u8> = Vec::new();
        bytes_vector.extend((70015 as i32).to_le_bytes());

        bytes_vector.extend(VarLenInt::new(2).to_bytes());
        let outpoint1 = Outpoint::new(outpoint_32_byte_array(), 4);
        let outpoint2 = Outpoint::new(outpoint_32_byte_array(), 4);
        let tx_in1 = TxIn::new(outpoint1, vec![1, 2, 3], 123);
        let tx_in2 = TxIn::new(outpoint2, vec![4, 5, 6], 456);
        bytes_vector.extend(tx_in1.to_bytes());
        bytes_vector.extend(tx_in2.to_bytes());

        bytes_vector.extend(VarLenInt::new(2).to_bytes());
        let tx_out1 = TxOut::new(1, vec![1, 2, 3]);
        let tx_out2 = TxOut::new(2, vec![4, 5, 6]);
        bytes_vector.extend(tx_out1.to_bytes());
        bytes_vector.extend(tx_out2.to_bytes());

        bytes_vector.extend((15 as u32).to_le_bytes());

        (bytes_vector, vec![tx_in1, tx_in2], vec![tx_out1, tx_out2])
    }

    fn transaction_expected_bytes_without_tx_in_and_tx_out() -> Vec<u8> {
        let mut bytes_vector: Vec<u8> = Vec::new();
        bytes_vector.extend((70015 as i32).to_le_bytes());
        bytes_vector.extend(VarLenInt::new(0).to_bytes());
        bytes_vector.extend(VarLenInt::new(0).to_bytes());
        bytes_vector.extend((0 as u32).to_le_bytes());
        bytes_vector
    }

    // Tests
    //=================================================================

    #[test]
    fn tx_in_test_1_to_bytes() {
        let tx_in = TxIn::new(
            Outpoint::new(outpoint_32_byte_array(), 4),
            vec![1, 2, 3, 4, 5],
            123,
        );
        let tx_in_bytes = tx_in.to_bytes();
        let tx_in_expected_bytes = tx_in_expected_bytes();
        assert_eq!(tx_in_bytes, tx_in_expected_bytes);
    }

    #[test]
    fn tx_test_2_from_bytes() {
        let tx_in = TxIn::new(
            Outpoint::new(outpoint_32_byte_array(), 4),
            vec![1, 2, 3, 4, 5],
            123,
        );
        let tx_in_bytes = tx_in.to_bytes();
        let tx_in_from_bytes = TxIn::from_bytes(&tx_in_bytes).unwrap();
        assert_eq!(tx_in, tx_in_from_bytes);
    }

    #[test]
    fn tx_out_test_1_to_bytes() {
        let tx_out = TxOut::new(5, vec![1, 2, 3, 4, 5]);

        assert_eq!(tx_out.to_bytes(), tx_out_expected_bytes());
    }

    #[test]
    fn tx_out_test_2_from_bytes() -> Result<(), TransactionError> {
        let expected_tx_out = TxOut::new(5, vec![1, 2, 3, 4, 5]);

        let tx_out = TxOut::from_bytes(&expected_tx_out.to_bytes())?;

        assert_eq!(expected_tx_out, tx_out);
        Ok(())
    }

    #[test]
    fn transaction_test_1_to_bytes_without_tx_in_and_tx_out() {
        let transaction = Transaction {
            version: 70015,
            tx_in_count: VarLenInt::new(0),
            tx_in: vec![],
            tx_out_count: VarLenInt::new(0),
            tx_out: vec![],
            lock_time: 0,
        };
        let transaction_bytes = transaction.to_bytes();
        let transaction_expected_bytes = transaction_expected_bytes_without_tx_in_and_tx_out();
        assert_eq!(transaction_bytes, transaction_expected_bytes);
    }

    #[test]
    fn transaction_test_2_to_bytes_with_tx_in_and_tx_out() {
        let (transaction_expected_bytes, tx_in, tx_out) =
            transaction_expected_bytes_with_tx_in_and_tx_out();
        let transaction = Transaction {
            version: 70015,
            tx_in_count: VarLenInt::new(2),
            tx_in,
            tx_out_count: VarLenInt::new(2),
            tx_out,
            lock_time: 15,
        };
        let transaction_bytes = transaction.to_bytes();
        assert_eq!(transaction_bytes, transaction_expected_bytes);
    }

    #[test]
    fn transaction_test_3_from_bytes_without_tx_in_and_tx_out() -> Result<(), TransactionError> {
        let transaction_bytes = transaction_expected_bytes_without_tx_in_and_tx_out();

        let transaction = Transaction::from_bytes(&transaction_bytes)?;

        assert_eq!(transaction_bytes, transaction.to_bytes());
        Ok(())
    }

    #[test]
    fn transaction_test_4_from_bytes_with_tx_in_and_tx_out() -> Result<(), TransactionError> {
        let (transaction_bytes, _tx_in, _tx_out) =
            transaction_expected_bytes_with_tx_in_and_tx_out();

        let transaction = Transaction::from_bytes(&transaction_bytes)?;

        assert_eq!(transaction_bytes, transaction.to_bytes());
        Ok(())
    }

/*
    fn get_bytes_from_hex(hex_string: &str)-> Vec<u8>{
        hex_string
            .as_bytes()
            .chunks(2)
            .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
            .collect::<Vec<u8>>()
    }

    fn real_transaction_sig_script() -> Vec<u8>{
        let hex_string = "473044022015e1ca708ca67db78c0513065e51165d22a8f79dc345ed5652b3689e55fb0e4702202a37273a155c3cb6b292d67460a836020443748bf498e5cf90d3e19be19a67c101210285664ba4fd95fb4c8f752fd07065b197a3b25a9a82ab6c1db877f3ec2ca43143"; // String en formato hexadecimal
        get_bytes_from_hex(hex_string)
    }

    fn real_outpoint_used() -> Outpoint{
        let hex_string = "5a23ad0ce6fe78458793baec33592a77175672f9f2e5216b1859d131c4252d0401000000"; // String en formato hexadecimal
        Outpoint::from_bytes(&get_bytes_from_hex(hex_string)).unwrap()
    }

    #[test]
    fn transaction_test_5_signatures() -> Result<(), TransactionError>{
        
        let amount = 0.01371463 * 100000000; 
        let unspent_balance = 51.43577627 * 100000000;        
        let fee = unspent_balance - amount - 51.42183664  * 100000000; // 0.000225 * 100000000

        Transaction::create(amount, fee, real_outpoint_used(), unspent_balance, pub_key, priv_key, address)
    }
*/

}
