use crate::blocks::transaction::*;
use crate::node::*;
use std::collections::HashMap;


impl Node {
    /// Creates the utxo set from the blockchain and returns it.
    /// Logs when error.
    pub fn create_utxo_set(&self) -> Result<HashMap<Vec<u8>, TxOut>, NodeError> {

        self.logger
            .log(format!("Initializing UTxO Set creation"));

        let mut utxo_set = HashMap::new();
        //p esto puede fallar pero por ahora le meto un unwrap
        let block_headers = self.get_block_headers().map_err(|_|NodeError::ErrorSharingReference)?;
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
                    let tx_out = TxOut::from_bytes(&tx_out.to_bytes()).map_err(|_|NodeError::ErrorGettingUtxo)?;

                    utxo_set.insert(tx_out_outpoint_bytes, tx_out);
                }
            }
        }

        self.logger
            .log(format!("UTxO Set created with {} UTxOs", utxo_set.len()));
        Ok(utxo_set)
    }
}

