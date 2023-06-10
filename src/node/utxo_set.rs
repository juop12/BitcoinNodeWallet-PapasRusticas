use crate::blocks::transaction::*;
use crate::node::*;
use std::collections::HashMap;

/*
impl Node {
    /// Creates the utxo set from the blockchain and returns it.
    /// Logs when error.
    pub fn create_utxo_set(&self) -> HashMap<Vec<u8>, &TxOut> {
        let mut utxo_set = HashMap::new();
        //p esto puede fallar pero por ahora le meto un unwrap
        let block_headers = self.get_block_headers().unwrap();
        let blockchain = self.get_blockchain().unwrap();
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
                for (index, tx_out) in tx.tx_out().iter().enumerate() {
                    let outpoint = Outpoint::new(tx.hash(), index as u32);
                    let tx_out_outpoint_bytes = outpoint.to_bytes();
                    utxo_set.insert(tx_out_outpoint_bytes, tx_out);
                }

                for tx_in in tx.tx_in().iter() {
                    let outpoint_bytes = tx_in.previous_output().to_bytes();

                    utxo_set.remove(&outpoint_bytes);
                }
            }
        }

        self.logger
            .log(format!("UTxO Set created with {} utxo", utxo_set.len()));
        utxo_set
    }
}
*/
