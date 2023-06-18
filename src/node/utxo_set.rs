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
                        for tx_in in tx.tx_in.iter() {
                            let outpoint_bytes = tx_in.previous_output.to_bytes();
        
                            utxo_set.remove(&outpoint_bytes);
                        }
        
                        for (index, tx_out) in tx.tx_out.iter().enumerate() {
                            //p ver si queremos nomas las p2pkh
                            if tx_out.pk_hash_under_p2pkh_protocol().is_some(){
                                let outpoint = Outpoint::new(tx.hash(), index as u32);
                                let tx_out_outpoint_bytes = outpoint.to_bytes();
                                let tx_out: TxOut = TxOut::from_bytes(&tx_out.to_bytes()).map_err(|_|NodeError::ErrorGettingUtxo)?;
            
                                utxo_set.insert(tx_out_outpoint_bytes, tx_out);
                            }
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

    pub fn update_utxo(&mut self)->Result<(), NodeError>{
        let mut block_hashes = Vec::new();
        match self.get_block_headers(){
            Ok(block_headers) => {
                for i in self.last_proccesed_block..block_headers.len(){
                    block_hashes.push(block_headers[i].hash());
                }
            },
            Err(error) => return Err(error),
        }
        
        let mut new_utxos = Vec::new();
        match self.get_blockchain(){
            Ok(blockchain) => {
                for hash in block_hashes{
                    if let Some(block) = blockchain.get(&hash){
                        let utxos = block.get_utxos_from(self.wallet_pk_hash);
                        new_utxos.extend(utxos);
                    }
                }
            },
            Err(error) => return Err(error),
        };
        for (key ,utxo) in new_utxos{
            self.insert_utxo(key, utxo)
        }
        self.last_proccesed_block += 1;
        Ok(())
    }

    fn insert_utxo(&mut self, key: Vec<u8>, tx_out: TxOut){
        self.balance += tx_out.value;
        self.utxo_set.insert(key, tx_out);
    }

    fn remove_utxo(&mut self, key: Vec<u8>) -> Option<TxOut>{
        let tx_out = self.utxo_set.remove(&key)?;
        self.balance -= tx_out.value;
        
        Some(tx_out)
    }
    ///-
    pub fn get_utxo_balance(&self, pk_hash: [u8; 20]) -> i64 {
        let mut balance = 0;

        for (_, tx_out) in &self.utxo_set{
            if let Some(p2pkh_hash) = tx_out.pk_hash_under_p2pkh_protocol(){
                if pk_hash == p2pkh_hash{
                    balance += tx_out.value;
                }
            }
        }
        
        return balance;
    }

    pub fn get_utxos_sum_up_to(&self, amount: i64) -> Result<(Vec<Outpoint>, i64), NodeError>{
        
        let mut unspent_balance = 0;
        let mut unspent_outpoint = Vec::new();
        let mut iter = self.utxo_set.iter();
        let curr = iter.next();
        let mut i = 0;
        
        while (unspent_balance < amount) && (curr.is_some()){
            if let Some((outpoint_bytes, utx_out)) = curr{
                if utx_out.belongs_to(self.wallet_pk_hash){
                    unspent_balance += utx_out.value;
                    let outpoint = Outpoint::from_bytes(&outpoint_bytes).map_err(|_| NodeError::ErrorSendingTransaction)?;
                    unspent_outpoint.push(outpoint);
                }
            } 
            iter.next();
            println!("utxo set length {}", self.utxo_set.len());
            i += 1;
            println!("iteracion: {}", i);
        };
        
        if unspent_balance < amount {
            return Err(NodeError::ErrorNotEnoughSatoshis);
        }
        
        Ok((unspent_outpoint, unspent_balance))
    }
}

