use crate::blocks::transaction::*;
use crate::node::*;
use std::collections::HashMap;

impl Node {
    /// Gets all utxos of the blockchain blocks
    fn _create_utxo_set(
        &self,
        block_headers: &Vec<BlockHeader>,
        utxo_set: &mut HashMap<Outpoint, TxOut>,
    ) -> Result<(), NodeError> {
        let blockchain = self
            .get_blockchain()
            .map_err(|_| NodeError::ErrorSharingReference)?;
        let starting_position = block_headers.len() - blockchain.len();

        for (index, header) in block_headers[starting_position..].iter().enumerate() {
            let hash = header.hash();
            let block = match blockchain.get(&hash) {
                Some(block) => block,
                None => {
                    self.logger.log(format!(
                        "Colud not find block number {} in create_utxo_set",
                        index + starting_position
                    ));
                    continue;
                }
            };

            update_utxo_set_with_transactions(block, utxo_set)?;
        }

        self.logger
            .log(format!("UTxO Set created with {} UTxOs", utxo_set.len()));

        Ok(())
    }

    /// Creates the utxo set from the blockchain and returns it.
    /// Logs when error.
    pub fn create_utxo_set(&mut self) -> Result<(), NodeError> {
        let initialization_str = "Initializing UTxO Set creation";
        self.log_and_send_to_ui(initialization_str);
 
        let mut utxo_set = HashMap::new();

        match self.get_block_headers() {
            Ok(block_headers) => self._create_utxo_set(&block_headers, &mut utxo_set)?,
            Err(_) => return Err(NodeError::ErrorSharingReference),
        };

        self.utxo_set = utxo_set;

        Ok(())
    }

    /// Gets UTXOS from any block that hasnt been yet proccesed
    fn get_utxos_from_unproccessed_blocks(
        &self,
        block_hash: &[u8; 32],
        blockchain: &HashMap<[u8; 32], Block>,
    ) -> Vec<(Outpoint, TxOut)> {
        let mut new_utxos = Vec::new();

        if let Some(block) = blockchain.get(block_hash) {
            let utxos = block.get_utxos();
            new_utxos.extend(utxos);
        }

        new_utxos
    }

    /// Takes out all the TXOUTS that are used as txin in a block
    fn get_spent_utxos_from_unproccesed_blocks(
        &self,
        block_hash: &[u8; 32],
        blockchain: &HashMap<[u8; 32], Block>,
    ) -> Vec<Outpoint> {
        let mut spent_utxos = Vec::new();

        if let Some(block) = blockchain.get(block_hash) {
            for tx in &block.transactions {
                for txin in &tx.tx_in {
                    let utxo_key = txin.previous_output;
                    if self.utxo_set.contains_key(&utxo_key) {
                        spent_utxos.push(utxo_key);
                    }
                }
            }
        }

        spent_utxos
    }

    // Proccesses all blocks received between the last time a block was proccessed and now
    pub fn update_utxo(
        &mut self,
        wallet_utxos: &mut HashMap<Outpoint, i64>,
    ) -> Result<(), NodeError> {
        let unproccesed_block_hash = match self.get_block_headers() {
            Ok(blockchain) => {
                if self.last_proccesed_block >= blockchain.len() {
                    return Ok(());
                }

                blockchain[self.last_proccesed_block].hash()
            }
            Err(error) => return Err(error),
        };

        let (spent_utxos, new_utxos) = match self.get_blockchain() {
            Ok(blockchain) => (
                self.get_spent_utxos_from_unproccesed_blocks(&unproccesed_block_hash, &blockchain),
                self.get_utxos_from_unproccessed_blocks(&unproccesed_block_hash, &blockchain),
            ),
            Err(error) => return Err(error),
        };

        for spent_utxo in spent_utxos {
            self.remove_utxo(spent_utxo, wallet_utxos);
        }
        for (key, utxo) in new_utxos {
            self.insert_utxo(key, utxo, wallet_utxos);
        }

        self.last_proccesed_block += 1;

        Ok(())
    }

    /// inserts the utxo, in the node and wallet, and updates balance
    fn insert_utxo(
        &mut self,
        key: Outpoint,
        tx_out: TxOut,
        wallet_utxos: &mut HashMap<Outpoint, i64>,
    ) {
        if tx_out.belongs_to(self.wallet_pk_hash) {
            self.balance += tx_out.value;
            wallet_utxos.insert(key, tx_out.value);
        }
        self.utxo_set.insert(key, tx_out);
    }

    /// Removes the utxo, in the node and wallet, and updates balance
    pub fn remove_utxo(
        &mut self,
        key: Outpoint,
        wallet_utxos: &mut HashMap<Outpoint, i64>,
    ) -> Option<TxOut> {
        let tx_out = self.utxo_set.remove(&key)?;
        if tx_out.belongs_to(self.wallet_pk_hash) {
            self.balance -= tx_out.value;
            wallet_utxos.remove(&key);
        }

        Some(tx_out)
    }

    /// Gets the utxos and their balance belonging to a given pkhash
    pub fn get_utxo_balance(&self, pk_hash: [u8; 20]) -> (HashMap<Outpoint, i64>, i64) {
        let mut balance = 0;
        let mut wallet_utxos = HashMap::new();

        for (outpoint, tx_out) in &self.utxo_set {
            if tx_out.belongs_to(pk_hash) {
                balance += tx_out.value;

                wallet_utxos.insert(*outpoint, tx_out.value);
            }
        }

        (wallet_utxos, balance)
    }

    /// Gets enough utxos whose values sum up to at least amount
    pub fn get_utxos_sum_up_to(&self, amount: i64) -> Result<(Vec<Outpoint>, i64), NodeError> {
        let mut unspent_balance = 0;
        let mut unspent_outpoint = Vec::new();

        for (outpoint, utx_out) in &self.utxo_set {
            if unspent_balance > amount {
                break;
            }

            if utx_out.belongs_to(self.wallet_pk_hash) {
                unspent_balance += utx_out.value;

                unspent_outpoint.push(*outpoint);
            }
        }

        if unspent_balance < amount {
            return Err(NodeError::ErrorNotEnoughSatoshis);
        }

        Ok((unspent_outpoint, unspent_balance))
    }
}

fn insert_new_utxo(
    tx_hash: [u8; 32],
    tx_out: &TxOut,
    index: usize,
    utxo_set: &mut HashMap<Outpoint, TxOut>,
) -> Result<(), NodeError> {
    let outpoint = Outpoint::new(tx_hash, index as u32);
    let tx_out: TxOut =
        TxOut::from_bytes(&tx_out.to_bytes()).map_err(|_| NodeError::ErrorGettingUtxo)?;

    utxo_set.insert(outpoint, tx_out);

    Ok(())
}

fn update_utxo_set_with_transactions(
    block: &Block,
    utxo_set: &mut HashMap<Outpoint, TxOut>,
) -> Result<(), NodeError> {
    for tx in block.get_transactions() {
        for tx_in in tx.tx_in.iter() {
            utxo_set.remove(&tx_in.previous_output);
        }

        for (index, tx_out) in tx.tx_out.iter().enumerate() {
            if tx_out.pk_hash_under_p2pkh_protocol().is_some() {
                insert_new_utxo(tx.hash(), tx_out, index, utxo_set)?;
            }
        }
    }

    Ok(())
}
