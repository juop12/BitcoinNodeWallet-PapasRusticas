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


    const VERSION: i32 = 70015;
    const LOCAL_HOST: [u8; 4] = [127, 0, 0, 1];
    const LOCAL_PORT: u16 = 1001; 
    const HASHEDGENESISBLOCK: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68,
        0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
        0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1,
        0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f,
    ];// 0x64 | [u8; 32] 


    #[test]
    fn test_1_valid_node_creates_a_set(){
        let logger = Logger::from_path("test_log.txt").unwrap();
        let mut node = Node::_new(VERSION, LOCAL_HOST, LOCAL_PORT, logger);

        node.ibd_send_get_block_headers_message(HASHEDGENESISBLOCK).unwrap();
        while node.receive_message(0).unwrap() != "headers\0\0\0\0\0" {

        }
        
        assert!(node.block_headers.len() == 2000);
        //descargar bloques y meterlos en blockchain


        assert!(node.create_utxo_set().len() > 0);
    }
}