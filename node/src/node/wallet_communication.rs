use crate::{
    blocks::{proof_of_transaction_included_in, HashPair, Transaction},
    messages::TxMessage,
    node::Node,
    utils::{btc_errors::NodeError, ui_communication_protocol::TxInfo, BlockInfo},
    wallet::Wallet,
};
use secp256k1::PublicKey;

impl Node {
    /// Returns a vec of TxInfo of all the tx in pending_tx that belong to a certain PubKey
    fn get_pending_tx_info_from(&self, pub_key: &PublicKey) -> Result<Vec<TxInfo>, NodeError> {
        let pending_tx = self.get_pending_tx()?;
        let mut wallet_pending_tx = Vec::new();

        for (_, tx) in pending_tx.iter() {
            let mut tx_in_amount = 0;

            for tx_in in &tx.tx_in {
                if tx_in.belongs_to(pub_key) {
                    if let Some(prev_tx_out) = self.utxo_set.get(&tx_in.previous_output) {
                        tx_in_amount -= prev_tx_out.value;
                    }
                }
            }

            let mut tx_out_amount = 0;

            for tx_out in &tx.tx_out {
                if tx_out.belongs_to(self.wallet_pk_hash) {
                    tx_out_amount += tx_out.value;
                }
            }

            if tx_out_amount + tx_in_amount != 0 {
                wallet_pending_tx.push(TxInfo::new(tx.hash(), tx_in_amount, tx_out_amount));
            }
        }

        Ok(wallet_pending_tx)
    }

    /// Updates a wallet pending_tx
    fn update_pending_tx(&self, wallet: &mut Wallet) -> Result<(), NodeError> {
        let pending_tx_info = self.get_pending_tx_info_from(&wallet.pub_key)?;
        wallet.update_pending_tx(pending_tx_info);

        Ok(())
    }

    /// Sets a new wallet as active for the node
    pub fn set_wallet(&mut self, wallet: &mut Wallet) -> Result<(), NodeError> {
        self.wallet_pk_hash = wallet.get_pk_hash();
        (wallet.utxos, wallet.balance) = self.get_utxo_balance(self.wallet_pk_hash);
        self.balance = wallet.balance;

        self.update_pending_tx(wallet)?;
        Ok(())
    }

    /// Updates the Node information and communicates it to the wallet
    pub fn update(&mut self, wallet: &mut Wallet) -> Result<(), NodeError> {
        match &self.peer_comunicator {
            Some(peer_communicator) => {
                if peer_communicator.disconected() {
                    return Err(NodeError::ErrorDisconectedFromBlockchain);
                }
            }
            None => return Err(NodeError::ErrorDisconectedFromBlockchain),
        }
        self.update_utxo(&mut wallet.utxos)?;
        wallet.balance = self.balance;
        self.update_pending_tx(wallet)?;
        Ok(())
    }

    fn get_block_info(&self, hash: [u8; 32], block_number: usize) -> Result<BlockInfo, NodeError> {
        let blockchain = self.get_blockchain()?;
        let block = match blockchain.get(&hash) {
            Some(block) => block,
            None => return Err(NodeError::ErrorFindingBlock),
        };

        let block_info = BlockInfo::new(block_number, block.get_header(), block.get_tx_hashes());

        Ok(block_info)
    }

    /// Gets the BlockInfo of a block corresponding to a block number
    pub fn get_block_info_from(&self, block_number: usize) -> Result<BlockInfo, NodeError> {
        let hash = match self.get_block_headers() {
            Ok(block_headers) => {
                if block_number > block_headers.len() {
                    return Err(NodeError::ErrorFindingBlock);
                }
                block_headers[block_number - 1].hash()
            }
            Err(error) => return Err(error),
        };

        self.get_block_info(hash, block_number)
    }

    /// Gets the block info corresponding to the last block of the blockchain
    pub fn get_last_block_info(&self) -> Result<BlockInfo, NodeError> {
        let last_block_number;
        let last_hash = match self.get_block_headers() {
            Ok(block_headers) => {
                last_block_number = block_headers.len();
                block_headers[last_block_number - 1].hash()
            }
            Err(error) => return Err(error),
        };

        self.get_block_info(last_hash, last_block_number)
    }

    /// Sends a transaction to all of its peers, puts it in pending t, and removes the used Utxos from the Utxo sets
    pub fn send_transaction(
        &mut self,
        wallet: &mut Wallet,
        transaction: Transaction,
    ) -> Result<(), NodeError> {
        let message = TxMessage::new(transaction);
        /*let mut sent = false;

        for (i, stream) in self.tcp_streams.iter_mut().enumerate() {
            self.logger.log(format!("mandando al peer{i}"));
            if message.send_to(stream).is_ok() {
                sent = true;
            }
        }

        if !sent {
            return Err(NodeError::ErrorSendingTransaction);
        }*/
        match &self.peer_comunicator {
            Some(peer_comunicator) => peer_comunicator
                .send_message(&message)
                .map_err(|_| NodeError::ErrorSendingTransaction)?,
            None => return Err(NodeError::ErrorSendingTransaction),
        };

        let transaction = message.tx;
        let transaction_hash = transaction.hash();
        let mut used_outpoints = Vec::new();
        match self.get_pending_tx() {
            Ok(mut pending_tx) => {
                pending_tx.insert(transaction_hash, transaction);

                if let Some(tx) = pending_tx.get(&transaction_hash) {
                    for txin in &tx.tx_in {
                        used_outpoints.push(txin.previous_output);
                    }
                }
            }
            Err(error) => return Err(error),
        }

        self.update_pending_tx(wallet)?;

        for outpoint in used_outpoints {
            self.remove_utxo(outpoint, &mut wallet.utxos);
        }

        self.logger.log("Se envio una transaccion".to_string());

        Ok(())
    }

    /// Returns the merkle proof of a given, block and transaction.
    pub fn get_merkle_tx_proof(
        &self,
        transaction_hash: [u8; 32],
        block_number: usize,
    ) -> Result<(Vec<HashPair>, [u8; 32]), NodeError> {
        let block_hash = match self.get_block_headers() {
            Ok(block_headers) => {
                if block_number - 1 < block_headers.len() {
                    block_headers[block_number - 1].hash()
                } else {
                    return Err(NodeError::ErrorFindingBlock);
                }
            }
            Err(error) => return Err(error),
        };
        match self.get_blockchain() {
            Ok(block_chain) => match block_chain.get(&block_hash) {
                Some(block) => Ok(proof_of_transaction_included_in(transaction_hash, block)),
                None => Err(NodeError::ErrorFindingBlock),
            },
            Err(error) => Err(error),
        }
    }
}
