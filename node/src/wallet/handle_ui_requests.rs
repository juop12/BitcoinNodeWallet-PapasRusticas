use crate::utils::ui_communication::{
    UIToWalletCommunication as UIRequest,
    WalletToUICommunication as UIResponse,
    WalletInfo,
};
use glib::{Sender as GlibSender};
use crate::{blocks::transaction::*, utils::WalletError};
use crate::node::Node;
use super::Wallet;


impl Wallet{
    pub fn handle_ui_request(self, node: &mut Node, request: UIRequest, sender_to_ui: &GlibSender<UIResponse>, program_running: &mut bool)-> Result<Wallet, WalletError>{
        match request{
            UIRequest::ChangeWallet(priv_key_string) => return self.handle_change_wallet(node, priv_key_string),
            UIRequest::CreateTx(amount, fee, address) => self.handle_create_tx(node, amount, fee, address)?,
            UIRequest::Update => self.handle_update(sender_to_ui)?,
            UIRequest::LastBlockInfo => todo!(),
            UIRequest::NextBlockInfo => todo!(),
            UIRequest::PrevBlockInfo => todo!(),
            UIRequest::EndOfProgram => *program_running = false,
        };
    
        Ok(self)
    }
    
    fn handle_update(&self, sender_to_ui: &GlibSender<UIResponse>) -> Result<(), WalletError>{
        let wallet_info = WalletInfo::from(self);
        
        sender_to_ui.send(UIResponse::WalletInfo(wallet_info)).map_err(|_| WalletError::ErrorSendingToUI)?;

        Ok(())
    }

    fn handle_change_wallet(&self, node: &mut Node, priv_key_string: String) -> Result<Wallet, WalletError>{
        let mut new_wallet = Wallet::from(priv_key_string)?;
        node.set_wallet(&mut new_wallet);

        Ok(new_wallet)
    }

    fn handle_create_tx(&self, node: &mut Node, amount: i64, fee: i64, receiver_address: String)-> Result<(), WalletError>{
        let address_bytes = bs58::decode(receiver_address).into_vec().map_err(|_| WalletError::ErrorSendingTx)?;
        let mut address: [u8;25] = [0;25];
        address.copy_from_slice(&address_bytes);
        self.create_transaction(node, amount, fee, address)?;
        
        Ok(())
    }
}