use crate::blocks::transaction::*;
use crate::node::*;
use std::collections::HashMap;


impl Node {
    ///-
    fn _create_utxo_set(&self, block_headers: &Vec<BlockHeader>, utxo_set: &mut HashMap<Vec<u8>, TxOut>) -> Result<(), NodeError>{

        let blockchain = self.get_blockchain().map_err(|_|NodeError::ErrorSharingReference)?;
        let starting_position = block_headers.len() - blockchain.len();
        
        //p arreglar esto
        for (index, header) in block_headers[starting_position..].iter().enumerate() {
            let hash = header.hash();
            let block = match blockchain.get(&hash) {
                Some(block) => block,
                None => {
                    self.logger.log(format!("Colud not find block number {} in create_utxo_set", index+starting_position));
                    continue;
                }
            };
        
            update_utxo_set_with_transactions(block, utxo_set)?;
        }
        
        self.logger.log(format!("UTxO Set created with {} UTxOs", utxo_set.len())); 

        Ok(())
    }

    /// Creates the utxo set from the blockchain and returns it.
    /// Logs when error.
    pub fn create_utxo_set(&mut self) -> Result<(), NodeError> {

        self.logger.log(format!("Initializing UTxO Set creation"));

        let mut utxo_set = HashMap::new();

        match self.get_block_headers(){
            Ok(block_headers) => self._create_utxo_set(&block_headers, &mut utxo_set)?,
            Err(_) => return Err(NodeError::ErrorSharingReference),
        };

        self.utxo_set = utxo_set;

        Ok(())
    }
    
    fn get_unproccesed_blocks_hashes(&mut self)->Result<Vec<[u8;32]>, NodeError>{
        let mut block_hashes = Vec::new();
        match self.get_block_headers(){
            Ok(block_headers) => {
                for i in self.last_proccesed_block..block_headers.len(){
                    block_hashes.push(block_headers[i].hash());
                }
                Ok(block_hashes)
            },
            Err(error) => Err(error),
        }
    }

    fn get_utxos_from_unproccessed_blocks(&self, block_hashes: &Vec<[u8;32]>, blockchain: &HashMap<[u8;32], Block>)->Vec<(Vec<u8>, TxOut)>{
        let mut new_utxos = Vec::new();
        
        for hash in block_hashes{
            if let Some(block) = blockchain.get(hash){
                let utxos = block.get_utxos();
                new_utxos.extend(utxos);
            }
        }
        
        new_utxos
    }

    fn get_spent_utxos_from_unproccesed_blocks(&self, block_hashes: &Vec<[u8;32]>, blockchain: &HashMap<[u8;32], Block>)-> Vec<Vec<u8>>{
        let mut spent_utxos = Vec::new();
        
        for hash in block_hashes{
            if let Some(block) = blockchain.get(hash){
                for tx in &block.transactions{
                    for txin in &tx.tx_in{
                        let utxo_key = txin.previous_output.to_bytes();
                        if self.utxo_set.contains_key(&utxo_key){
                            spent_utxos.push(utxo_key);
                        }
                    }
                }
            }
        }

        spent_utxos
    }

    // Proccesses all blocks received between the last time a block was proccessed and now
    pub fn update_utxo(&mut self)->Result<(), NodeError>{
        
        let block_hashes = self.get_unproccesed_blocks_hashes()?;
        if block_hashes.is_empty(){
            return Ok(());
        }
        
        let (spent_utxos, new_utxos) = match self.get_blockchain(){
            Ok(blockchain) => {
                (self.get_spent_utxos_from_unproccesed_blocks(&block_hashes, &blockchain),
                self.get_utxos_from_unproccessed_blocks(&block_hashes, &blockchain))
            },
            Err(error) => return Err(error),
        };

        for spent_utxo in spent_utxos{
            self.remove_utxo(spent_utxo);
        }
        for (key ,utxo) in new_utxos{
            self.insert_utxo(key, utxo)
        }

        self.last_proccesed_block += block_hashes.len();
        
        Ok(())
    }

    fn insert_utxo(&mut self, key: Vec<u8>, tx_out: TxOut){
        if tx_out.belongs_to(self.wallet_pk_hash){   
            self.balance += tx_out.value;
        }
        self.utxo_set.insert(key, tx_out);
    }

    pub fn remove_utxo(&mut self, key: Vec<u8>) -> Option<TxOut>{
        let tx_out = self.utxo_set.remove(&key)?;
        if tx_out.belongs_to(self.wallet_pk_hash){
            self.balance -= tx_out.value;
        }
        
        Some(tx_out)
    }
    
    ///-
    pub fn get_utxo_balance(&self, pk_hash: [u8; 20]) -> i64 {
        let mut balance = 0;

        for (_, tx_out) in &self.utxo_set{
            if tx_out.belongs_to(pk_hash){
                balance += tx_out.value;
            }
        }
        
        return balance;
    }

    ///-
    pub fn get_utxos_sum_up_to(&self, amount: i64) -> Result<(Vec<Outpoint>, i64), NodeError>{
        
        let mut unspent_balance = 0;
        let mut unspent_outpoint = Vec::new();
        
        for (outpoint_bytes, utx_out) in &self.utxo_set{
            if unspent_balance > amount{
                break;
            }

            if utx_out.belongs_to(self.wallet_pk_hash){
                unspent_balance += utx_out.value;

                let outpoint = Outpoint::from_bytes(&outpoint_bytes).map_err(|_| NodeError::ErrorSendingTransaction)?;
                unspent_outpoint.push(outpoint);
            }
        }
        
        if unspent_balance < amount {
            return Err(NodeError::ErrorNotEnoughSatoshis);
        }
        
        Ok((unspent_outpoint, unspent_balance))
    }
}

///-
fn insert_new_utxo(tx_hash: [u8; 32], tx_out: &TxOut, index: usize, utxo_set: &mut HashMap<Vec<u8>, TxOut>) -> Result<(), NodeError>{
    let outpoint = Outpoint::new(tx_hash, index as u32);
    let tx_out_outpoint_bytes = outpoint.to_bytes();
    let tx_out: TxOut = TxOut::from_bytes(&tx_out.to_bytes()).map_err(|_|NodeError::ErrorGettingUtxo)?;

    utxo_set.insert(tx_out_outpoint_bytes, tx_out);

    Ok(())
}

///-
fn update_utxo_set_with_transactions(block: &Block, utxo_set: &mut HashMap<Vec<u8>, TxOut>) -> Result<(), NodeError> {
    for tx in block.get_transactions() {
        for tx_in in tx.tx_in.iter() {
            let outpoint_bytes = tx_in.previous_output.to_bytes();

            utxo_set.remove(&outpoint_bytes);
        }

        for (index, tx_out) in tx.tx_out.iter().enumerate() {
            //p ver si queremos nomas las p2pkh
            if tx_out.pk_hash_under_p2pkh_protocol().is_some(){
                insert_new_utxo(tx.hash(), tx_out, index, utxo_set)?;
            }
        }
    }

    Ok(())
}