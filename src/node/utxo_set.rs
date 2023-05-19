use crate::blocks::transaction::*;
use std::collections::HashMap;
use crate::node::*;

impl Node {


    pub fn create_utxo_set(&self) -> Option<HashMap<Vec<u8>, &TxOut>>{

        let mut utxo_set = HashMap::new();

        let starting_position = self.block_headers.len() - self.blockchain.len();

        for header in &self.block_headers[starting_position..]{
            let hash = header.hash();
            let block = self.blockchain.get(&hash)?;

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

    /* 
        for block in self.blockchain.values(){
            for tx in block.get_transactions(){
                for (i, tx_out) in tx.tx_out().iter().enumerate(){
                    let hash = tx.hash();
                    let outpoint = Outpoint::new(hash, i as u32);
                    let outpoint_bytes = outpoint.to_bytes();
                    utxo_set.insert(outpoint_bytes, tx_out);
                }
            }
        }
        
        for block in self.blockchain.values(){
            for tx in block.get_transactions(){
                for tx_in in tx.tx_in().iter(){

                    let outpoint_bytes = tx_in.previous_output().to_bytes();

                    utxo_set.remove(&outpoint_bytes);
                }
            }
        } 
    */
        Some(utxo_set)
    }
}


#[cfg(test)]
mod tests{
    use super::*;


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

        let utxo_set = node.create_utxo_set().unwrap();

        println!("utx_set Len:: {}\n\n", utxo_set.len());
        println!("utx_set:: {:?}\n\n", utxo_set);

        assert!(utxo_set.len() > 0);
        Ok(())
    }
}
