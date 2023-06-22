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

    pub fn create_transaction(&self, node: &mut Node, amount: i64, fee: i64, address: [u8; 25]) -> Result<(), NodeError>{
        
        let (unspent_outpoints, unspent_balance) = node.get_utxos_sum_up_to(amount + fee)?;
        if unspent_outpoints.len() < 2{
            panic!("not happening");
        }
        println!("Se agarraron {} outpoints", unspent_outpoints.len());

        let transaction = Transaction::create(amount, fee, unspent_outpoints, unspent_balance, self.pub_key, self.priv_key, address)
            .map_err(|_| NodeError::ErrorSendingTransaction)?;
        
        node.logger.log(format!("se empezo a enviar la transaccion"));
        node.send_transaction(transaction)?;
        
        Ok(())
    }
}