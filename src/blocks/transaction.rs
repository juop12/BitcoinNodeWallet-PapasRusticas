use bitcoin_hashes::{sha256d, Hash};
use crate::utils::{
    variable_length_integer::VarLenInt,
    btc_errors::TransactionError,
};


const MIN_BYTES_TX_IN :usize = 41;
const MIN_BYTES_TX_OUT :usize = 9;
const MIN_BYTES_TRANSACTION: usize = 10;
const OUTPOINT_BYTES :usize = 36;


/// Struct that represents the Outpoint, that is used in the TxIn struct.
#[derive(Debug, PartialEq)]
pub struct Outpoint {
    hash: [u8; 32],
    index: u32,
}

/// Struct that represents the TxIn used in the struct Transaction.
#[derive(Debug, PartialEq)]
pub struct TxIn {
    previous_output: Outpoint,
    script_length: VarLenInt,
    signature_script: Vec<u8>,
    sequence: u32, // u32::MAX; Se usa el maximo u32.
}

/// Struct that represents the TxOut used in the struct Transaction
#[derive(Debug, PartialEq)]
pub struct TxOut {
    value: i64,
    pk_script_length: VarLenInt,
    pk_script: Vec<u8>,
}

/// Implementation of a Transaction in a Bitcoin block.
#[derive(Debug, PartialEq)]
pub struct Transaction {
    version: i32,
    tx_in_count: VarLenInt,
    tx_in: Vec<TxIn>,
    tx_out_count: VarLenInt,
    tx_out: Vec<TxOut>,
    lock_time: u32, // siempre va 0.
}

impl Outpoint{
    ///Creates a new Outpoint
    pub fn new(hash: [u8;32], index: u32) -> Outpoint{
        Outpoint{hash, index}
    }

    ///Returns the contents of Outpoint as a bytes vecotr
    pub fn to_bytes(&self)-> Vec<u8>{
        let mut bytes = Vec::from(self.hash);
        bytes.extend(self.index.to_le_bytes());
        bytes
    }
    
    ///If the bytes given can form a valid Outpoint, it creates it, if not returns error
    fn from_bytes(slice :&[u8])->Result<Outpoint, TransactionError>{
        if slice.len() != OUTPOINT_BYTES{
            return Err(TransactionError::ErrorCreatingOutpointFromBytes);
        }
        let (hash_bytes, index_bytes) = slice.split_at(32);
        let hash :[u8;32] = match hash_bytes.try_into(){
            Ok(hash) => hash,
            Err(_) => return Err(TransactionError::ErrorCreatingOutpointFromBytes),
        };
        let index :u32 = match index_bytes.try_into(){
            Ok(index) => u32::from_le_bytes(index),
            Err(_) => return Err(TransactionError::ErrorCreatingOutpointFromBytes),
        };
        
        Ok(Outpoint::new(hash, index))
    }
}

impl TxOut {
    pub fn new(value: i64, pk_script: Vec<u8>) -> TxOut {
        let pk_script_length = VarLenInt::new(pk_script.len());
        TxOut {
            value,
            pk_script_length,
            pk_script,
        }
    }

    ///Returns the contents of Outpoint as a bytes vecotr
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.value.to_le_bytes());
        bytes_vector.extend_from_slice(&self.pk_script_length.to_bytes());
        bytes_vector.extend_from_slice(&self.pk_script);
        bytes_vector
    }

    /// If the bytes given can form a valid TxOut, it creates it, if not returns error.
    /// It is important to note that not all bytes passed will be used. The function 
    /// will only use the bytes needed to create the TxOut
    fn from_bytes(slice: &[u8]) -> Result<TxOut, TransactionError> {
        if slice.len() < MIN_BYTES_TX_OUT{
            return Err(TransactionError::ErrorCreatingTxOutFromBytes);
        }

        let value = match slice[0..8].try_into().ok() {
            Some(value) => i64::from_le_bytes(value),
            None => return Err(TransactionError::ErrorCreatingTxOutFromBytes),
        };
        let pk_script_length = VarLenInt::from_bytes(&slice[8..]);
        let (_left_bytes, slice) = slice.split_at(8+pk_script_length.amount_of_bytes());
        let pk_script = slice[0 .. pk_script_length.to_usize()].to_vec();

        Ok(TxOut {
            value,
            pk_script_length,
            pk_script,
        })
    }

    fn amount_of_bytes(&self) -> usize{
        self.to_bytes().len()
    }
}

impl TxIn{
    /// Creates a new TxIn
    pub fn new(previous_output: Outpoint, signature_script: Vec<u8>, sequence: u32) -> TxIn {
        let script_length = VarLenInt::new(signature_script.len());
        TxIn{
            previous_output,
            script_length,
            signature_script,
            sequence,
        }
    }

    /// Returns the contents of TxIn as a bytes vector
    fn to_bytes(&self) -> Vec<u8>{
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
        if slice.len() < MIN_BYTES_TX_IN{
            return Err(TransactionError::ErrorCreatingTxInFromBytes);
        }
        let (prev_out_bytes, slice) = slice.split_at(OUTPOINT_BYTES);
        let previous_output = Outpoint::from_bytes(prev_out_bytes)?;
        let script_length = VarLenInt::from_bytes(&slice);
        let (_script_length_bytes, slice) = slice.split_at(script_length.amount_of_bytes());
        if slice.len() < script_length.to_usize() + 4{
            return Err(TransactionError::ErrorCreatingTxInFromBytes);
        }
        let (signature_script_bytes, slice) = slice.split_at(script_length.to_usize());
        let signature_script = signature_script_bytes.to_vec();
        let (sequence_bytes, _slice) = slice.split_at(4);
        let sequence = match sequence_bytes.try_into(){
            Ok(sequence_bytes) => u32::from_le_bytes(sequence_bytes),
            Err(_) => return Err(TransactionError::ErrorCreatingTxInFromBytes),
        };

        Ok(TxIn{
            previous_output,
            script_length,
            signature_script,
            sequence,
        })
    }

    /// Returns the amount of bytes needed to represent the TxIn
    fn amount_of_bytes(&self) -> usize{
        self.to_bytes().len()
    }

    pub fn previous_output(&self) -> &Outpoint{
        &self.previous_output
    }
}

impl Transaction {
    /// Creates a new Transaction
    pub fn new(version: i32, tx_in_vector: Vec<TxIn>, tx_out_vector: Vec<TxOut>, lock_time: u32) -> Transaction {
        Transaction {
            version,
            tx_in_count: VarLenInt::new(tx_in_vector.len()),
            tx_in: tx_in_vector,
            tx_out_count: VarLenInt::new(tx_out_vector.len()),
            tx_out: tx_out_vector,
            lock_time,
        }
    }

    /// Returns the contents of Transaction as a bytes vector
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes_vector = Vec::new();
        bytes_vector.extend_from_slice(&self.version.to_le_bytes());
        bytes_vector.extend_from_slice(&self.tx_in_count.to_bytes());
        for tx in &self.tx_in{
            bytes_vector.extend_from_slice(&tx.to_bytes());
        }
        bytes_vector.extend_from_slice(&self.tx_out_count.to_bytes());
        for tx in &self.tx_out{
            bytes_vector.extend_from_slice(&tx.to_bytes());
        }
        bytes_vector.extend_from_slice(&self.lock_time.to_le_bytes());
        bytes_vector
    }
    
    /// If the bytes given can form a valid Transaction, it creates it, if not returns error.
    pub fn from_bytes(slice: &[u8])-> Result<Transaction,TransactionError>{
        if slice.len() < MIN_BYTES_TRANSACTION{
            return Err(TransactionError::ErrorCreatingTxInFromBytes);
        }
        match Self::_from_bytes(slice){
            Some(transaction) => Ok(transaction),
            None => Err(TransactionError::ErrorCreatingTxInFromBytes),
        }
    }

    /// It receives the slice of bytes and checks if it can form a valid Transaction by converting the bytes into
    /// the corresponding fields. If it can, it returns the Transaction, if not, it returns None
    fn _from_bytes(slice: &[u8]) -> Option<Transaction> {
        let version = i32::from_le_bytes(slice[0..4].try_into().ok()?);
        let tx_in_count = VarLenInt::from_bytes(&slice[4..]);
        let (mut _used_bytes, mut slice) = slice.split_at(4 + tx_in_count.amount_of_bytes());

        let mut tx_in :Vec<TxIn> = Vec::new();
        for _ in 0..tx_in_count.to_usize(){
            let tx = TxIn::from_bytes(slice).ok()?;
            (_used_bytes, slice) = slice.split_at(tx.amount_of_bytes());
            tx_in.push(tx);
        }

        let tx_out_count = VarLenInt::from_bytes(slice);
        (_used_bytes, slice) = slice.split_at(tx_out_count.amount_of_bytes());

        let mut tx_out :Vec<TxOut> = Vec::new();
        for _ in 0..tx_out_count.to_usize(){
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

    pub fn ammount_of_bytes(&self) -> usize{
        self.to_bytes().len()
    }

    pub fn hash(&self) -> [u8;32]{
        *sha256d::Hash::hash(&self.to_bytes()).as_byte_array()
    }

    pub fn tx_out(&self) -> &Vec<TxOut>{
        &self.tx_out
    }

    pub fn tx_in(&self) -> &Vec<TxIn>{
        &self.tx_in
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    // Auxiliar functions
    //=================================================================

    fn outpoint_32_byte_array() -> [u8; 32] {
        [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31]
    }

    fn tx_in_expected_bytes() -> Vec<u8> {
        let mut bytes_vector: Vec<u8> = Vec::new();
        let outpoint = Outpoint::new(outpoint_32_byte_array(),4);
        bytes_vector.extend_from_slice(&outpoint.to_bytes());
        bytes_vector.extend_from_slice(&VarLenInt::new(5).to_bytes());
        bytes_vector.extend_from_slice(&[1,2,3,4,5]);
        bytes_vector.extend_from_slice(&(123 as u32).to_le_bytes());
        bytes_vector
    }

    fn tx_out_expected_bytes()-> Vec<u8>{
        let mut bytes_vector: Vec<u8> = Vec::new();
        bytes_vector.extend((5 as i64).to_le_bytes());
        bytes_vector.extend(VarLenInt::new(5).to_bytes());
        bytes_vector.extend([1,2,3,4,5]);
        bytes_vector
    }
    
    fn transaction_expected_bytes_with_tx_in_and_tx_out() -> (Vec<u8>, Vec<TxIn>, Vec<TxOut>) {
        let mut bytes_vector: Vec<u8> = Vec::new();
        bytes_vector.extend((70015 as i32).to_le_bytes());
    
        bytes_vector.extend(VarLenInt::new(2).to_bytes());    
        let outpoint1 = Outpoint::new(outpoint_32_byte_array(),4);
        let outpoint2 = Outpoint::new(outpoint_32_byte_array(),4);
        let tx_in1 = TxIn::new(outpoint1, vec![1,2,3],123);
        let tx_in2 = TxIn::new(outpoint2, vec![4,5,6],456);
        bytes_vector.extend(tx_in1.to_bytes());
        bytes_vector.extend(tx_in2.to_bytes());

        bytes_vector.extend(VarLenInt::new(2).to_bytes());
        let tx_out1 = TxOut::new(1, vec![1,2,3]);
        let tx_out2 = TxOut::new(2, vec![4,5,6]);
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
            Outpoint::new(
                outpoint_32_byte_array(),
                4,
            ),
            vec![1,2,3,4,5],
            123,
        );
        let tx_in_bytes = tx_in.to_bytes();
        let tx_in_expected_bytes = tx_in_expected_bytes();
        assert_eq!(tx_in_bytes, tx_in_expected_bytes);
    }

    #[test]
    fn tx_test_2_from_bytes() {
        let tx_in = TxIn::new(
            Outpoint::new(
                outpoint_32_byte_array(),
                4,
            ),
            vec![1,2,3,4,5],
            123,
        );
        let tx_in_bytes = tx_in.to_bytes();
        let tx_in_from_bytes = TxIn::from_bytes(&tx_in_bytes).unwrap();
        assert_eq!(tx_in, tx_in_from_bytes);
    }

    #[test]
    fn tx_out_test_1_to_bytes(){
        let tx_out = TxOut::new(5, vec![1,2,3,4,5]);

        assert_eq!(tx_out.to_bytes(), tx_out_expected_bytes());
    }

    #[test]
    fn tx_out_test_2_from_bytes()-> Result<(), TransactionError>{
        let expected_tx_out = TxOut::new(5, vec![1,2,3,4,5]);
        
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
        let (transaction_expected_bytes, tx_in, tx_out) = transaction_expected_bytes_with_tx_in_and_tx_out();
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
        let (transaction_bytes, _tx_in, _tx_out) = transaction_expected_bytes_with_tx_in_and_tx_out();
        
        let transaction = Transaction::from_bytes(&transaction_bytes)?;

        assert_eq!(transaction_bytes, transaction.to_bytes());
        Ok(())
    }
}       