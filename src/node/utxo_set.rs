use crate::blocks::transaction::*;
use std::collections::HashMap;
use crate::node::*;


impl Node {

    /// Creates the utxo set from the blockchain and returns it.
    /// Logs when error.
    pub fn create_utxo_set(&self) -> HashMap<Vec<u8>, &TxOut>{

        let mut utxo_set = HashMap::new();

        let starting_position = self.block_headers.len() - self.blockchain.len();

        for header in &self.block_headers[starting_position..]{
            let hash = header.hash();
            let block = match self.blockchain.get(&hash){
                Some(block) => block,
                None => {
                    self.logger.log(String::from("Colud not find block in create_utxo_set"));
                    continue;
                }
            };

            for tx in block.get_transactions(){
                for (index, tx_out) in tx.tx_out().iter().enumerate(){
                    
                    let outpoint = Outpoint::new(tx.hash(), index as u32);
                    let tx_out_outpoint_bytes = outpoint.to_bytes();
                    utxo_set.insert(tx_out_outpoint_bytes, tx_out);
                }

                for tx_in in tx.tx_in().iter(){
                    
                    let outpoint_bytes = tx_in.previous_output().to_bytes();

                    utxo_set.remove(&outpoint_bytes);
                }
            }
        }

        self.logger.log(format!("UTxO Set created with {} utxo", utxo_set.len()));
        utxo_set
    }
}
