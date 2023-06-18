use bitcoin_hashes::{hash160, Hash};
use crate::blocks::transaction::*;
use crate::node::*;
use crate::utils::btc_errors::NodeError;

pub struct Wallet{
    pub_key: [u8; 33],
    priv_key: [u8; 32],
    balance: i64,
    pending_balance: i64,
}

impl Wallet{
    pub fn new(pub_key: [u8; 33], priv_key: [u8; 32]) -> Wallet{

        //let balance = node.get_utxo_balance(pub_key);
        //let pending_balance = node.get_pending_balance(pub_key);

        Wallet {
            pub_key,
            priv_key,
            balance: 0,
            pending_balance: 0,
        }
    }

    pub fn get_pk_hash(&self) -> [u8; 20]{
        hash160::Hash::hash(self.pub_key.as_slice()).to_byte_array()
    }

    pub fn update(&mut self, balance: i64){
        self.balance = balance;
    }


    //pub struct TxIn {
    //    previous_output: Outpoint,    (Clave del UTxO.from_bytes())
    //    script_length: VarLenInt,     (lo vemos)
    //    signature_script: Vec<u8>,    (hay que firmar la nosabemosque)  //nos falta
    //    sequence: u32, // u32::MAX; Se usa el maximo u32. (0xffffffff)
    //}

    //pub struct TxOut {
    //    pub value: i64,                       (segun corresponda)
    //    pub pk_script_length: VarLenInt,      (lo vemos)
    //    pub pk_script: Vec<u8>,               (del destinatario a partir del adress creo) //masomenos
    //}

    //firmar
    //  tenemos la raw transaction
    //  1- metemos en el campo sig_script el pk_script (si habia algo se saca, para chequear)
    //  2- metemos el hash_type al final de la raw tx, probablemente SIGHASH_ALL(01000000)
    //  3-  z = int::from_big_endian  hash256(modified_transaccion.to_bytes)
    //  4- der = private_key.sign(z).der()
    //  sig = der + SIGHASH_ALL.to_bytes(1, 'big')  The signature is actually a combination of the DER signature and the hash type, (suma) which is SIGHASH_ALL in our case
    //  5- sec = private_key.point.sec() // ES LA PUBKEY COMPRESSED DE 33 BYTES!!!
    //  6- sig_script = [varlenInt(sig), sig, Varlenint(sec), sec]

    pub fn create_transaction(&self, node: &mut Node, amount: i64, fee: i64, address: [u8; 25]) -> Result<(), NodeError>{
        
        let (unspent_outpoints, unspent_balance) = node.get_utxos_sum_up_to(amount)?;
        println!("el unspent ballance es {}", unspent_balance); //p
        
        // hacer el cambio de base de base58 a base 16 del address. Checkear si recibimos address: [u8; 25] (hexa) o address: [u8; 17] (b58 -> 34 caracteres)

        let transaction = Transaction::create(amount, fee, unspent_outpoints, unspent_balance, self.pub_key, self.priv_key, address)
            .map_err(|_| NodeError::ErrorSendingTransaction)?;
        
        //p
        println!("se empezo a enviar la transaccion"); //p
        node.send_transaction(transaction)?;
        
        Ok(())
    }
}