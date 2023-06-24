use crate::{node::Node,
    wallet::Wallet,
    utils::{btc_errors::NodeError, ui_communication_protocol::TxInfo},
    blocks::Transaction,
    messages::{message_trait::*, TxMessage}
};
use secp256k1::PublicKey;

impl Node {
    ///-
    fn get_pending_tx_info_from(&self, pub_key: &PublicKey) -> Result<Vec<TxInfo>, NodeError>{

        let pending_tx = self.get_pending_tx()?;
        let mut wallet_pending_tx = Vec::new();

        for (_, tx) in pending_tx.iter(){
            let mut tx_in_amount = 0;
            let amount;
            
            for tx_in in &tx.tx_in{
                if tx_in.belongs_to(&pub_key){
                    if let Some(prev_tx_out) = self.utxo_set.get(&tx_in.previous_output){
                        tx_in_amount += prev_tx_out.value;            
                    }
                }
            }
            
            let mut tx_out_amount = 0;

            for tx_out in &tx.tx_out{
                if tx_out.belongs_to(self.wallet_pk_hash){
                    tx_out_amount += tx_out.value;
                }
            }
            
            if tx_in_amount == 0 {
                amount = tx_out_amount;
            } else {
                amount =  tx_out_amount - tx_in_amount;
            }

            wallet_pending_tx.push(TxInfo::new(tx.hash(), amount));
        }

        Ok(wallet_pending_tx)
    }
    
    fn update_pending_tx(&self, wallet: &mut Wallet) -> Result<(), NodeError>{
        
        let pending_tx_info = self.get_pending_tx_info_from(&wallet.pub_key)?;
        wallet.update_pending_tx(pending_tx_info);
        
        Ok(())
    }

    ///-
    pub fn set_wallet(&mut self, wallet: &mut Wallet) -> Result<(), NodeError>{
        self.wallet_pk_hash = wallet.get_pk_hash();
        (wallet.utxos, wallet.balance) = self.get_utxo_balance(self.wallet_pk_hash);
        self.balance = wallet.balance;

        self.update_pending_tx(wallet)?;
        println!("balance {}",self.balance); //p
        Ok(())
    }

    ///-
    pub fn update(&mut self, wallet: &mut Wallet)-> Result<(), NodeError>{
        self.update_utxo(&mut wallet.utxos)?;
        wallet.balance = self.balance;
        self.update_pending_tx(wallet)?;
        
        Ok(())
    }

    ///-
    pub fn send_transaction(&mut self, transaction: Transaction) -> Result<(), NodeError>{
        
        let message = TxMessage::new(transaction);
        let mut sent = false;
        
        for (i, stream) in self.tcp_streams.iter_mut().enumerate() {
            self.logger.log(format!("mandando al peer{i}"));
            println!("mandando al peer{i}");
            if message.send_to(stream).is_ok(){
                sent = true;
            }
        }
        
        if !sent {
            return Err(NodeError::ErrorSendingTransaction);
        }
        
        let transaction = message.tx;
        
        for txin in &transaction.tx_in{
            self.utxo_set.remove(&txin.previous_output);
        }

        self.get_pending_tx()?.insert(transaction.hash(), transaction);
        
        self.logger.log(format!("Se envio una transaccion"));
        println!("Se envio una transaccion"); //p
        
        Ok(())
    }
}