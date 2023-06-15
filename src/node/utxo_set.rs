use crate::blocks::transaction::*;
use crate::node::*;
use std::collections::HashMap;
use bitcoin_hashes::{hash160, Hash};


impl Node {
    /// Creates the utxo set from the blockchain and returns it.
    /// Logs when error.
    pub fn create_utxo_set(&mut self) -> Result<(), NodeError> {

        self.logger
            .log(format!("Initializing UTxO Set creation"));

        let mut utxo_set = HashMap::new();
        
        //p esto puede fallar pero por ahora le meto un unwrap
        match self.get_block_headers(){
            Ok(block_headers) => {
                let blockchain = self.get_blockchain().map_err(|_|NodeError::ErrorSharingReference)?;
                let starting_position = block_headers.len() - blockchain.len();
        
                for header in &block_headers[starting_position..] {
                    let hash = header.hash();
                    let block = match blockchain.get(&hash) {
                        Some(block) => block,
                        None => {
                            self.logger
                                .log(String::from("Colud not find block in create_utxo_set"));
                            continue;
                        }
                    };
        
                    for tx in block.get_transactions() {
                        for tx_in in tx.tx_in().iter() {
                            let outpoint_bytes = tx_in.previous_output().to_bytes();
        
                            utxo_set.remove(&outpoint_bytes);
                        }
        
                        for (index, tx_out) in tx.tx_out().iter().enumerate() {
                            let outpoint = Outpoint::new(tx.hash(), index as u32);
                            let tx_out_outpoint_bytes = outpoint.to_bytes();
                            let tx_out: TxOut = TxOut::from_bytes(&tx_out.to_bytes()).map_err(|_|NodeError::ErrorGettingUtxo)?;
        
                            utxo_set.insert(tx_out_outpoint_bytes, tx_out);
                        }
                    }
                }
        
                self.logger
                    .log(format!("UTxO Set created with {} UTxOs", utxo_set.len())); 
            }
            Err(_) => return Err(NodeError::ErrorSharingReference),
        };
        
        self.utxo_set = utxo_set;
        Ok(())
    }


    ///-
    pub fn get_utxo_balance(&self, pub_key: [u8; 33]) -> i64 {
        let mut balance = 0;
        let pk_hash = hash160::Hash::hash(&pub_key.as_slice());

        for (_, tx_out) in self.get_utxo_set(){
            if pk_hash.to_byte_array() == tx_out.get_pk_script()[3..23]{
                balance += *tx_out.get_value();
            }
        }

        return balance;
    }
}

