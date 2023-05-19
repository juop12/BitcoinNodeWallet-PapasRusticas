use crate::blocks::transaction::*;
use std::collections::HashMap;
use crate::node::*;

impl Node {


    pub fn create_utxo_set(&self) -> HashMap<Vec<u8>, &TxOut>{

        let mut utxo_set = HashMap::new();

        for block in &self.blockchain{
            for tx in block.transactions(){
                for (i, tx_out) in tx.tx_out().iter().enumerate(){
                    let hash = tx.hash();
                    let outpoint = Outpoint::new(hash, i as u32);
                    let outpoint_bytes = outpoint.to_bytes();
                    utxo_set.insert(outpoint_bytes, tx_out);
                }
            }
        }
        
        for block in &self.blockchain{
            for tx in block.transactions(){
                for tx_in in tx.tx_in().iter(){

                    let outpoint_bytes = tx_in.previous_output().to_bytes();

                    utxo_set.remove(&outpoint_bytes);
                }
            }
        }

        utxo_set
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    use crate::node::data_handler::NodeDataHandler;
    use std::{
        sync::{Arc, Mutex},
    };


    #[test]
    fn test_1_valid_node_creates_a_set()-> Result<(), NodeError>{
        let config = Config {
            version: 70015,
            dns_port: 18333,
            local_host: [127,0,0,1],
            local_port: 1001,
            log_path: String::from("src/node_log.txt"),
            begin_time: 1681084800,
        };
        let mut node = Node::new(config)?;
        node.initial_block_download()?;
        let utxo_set = node.create_utxo_set();
        assert!(utxo_set.len() > 0);
        Ok(())
    }
}