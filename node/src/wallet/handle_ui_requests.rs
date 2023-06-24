use crate::utils::NodeError;
use crate::utils::ui_communication_protocol::{
    UIToWalletCommunication as UIRequest,
    WalletToUICommunication as UIResponse,
    WalletInfo,
};
use glib::{Sender as GlibSender};
use crate::{blocks::transaction::*, utils::WalletError};
use crate::node::Node;
use super::Wallet;


impl Wallet{
    pub fn handle_ui_request(mut self, node: &mut Node, request: UIRequest, sender_to_ui: &GlibSender<UIResponse>, program_running: &mut bool)-> Result<Wallet, WalletError>{
        match request{
            UIRequest::ChangeWallet(priv_key_string) => return self.handle_change_wallet(node, priv_key_string, sender_to_ui),
            UIRequest::CreateTx(amount, fee, address) => self.handle_create_tx(node, amount, fee, address)?,
            UIRequest::Update => self.handle_update(sender_to_ui)?,
            UIRequest::LastBlockInfo => self.handle_last_block_info(node, sender_to_ui)?,
            UIRequest::NextBlockInfo => self.handle_get_block_info(node, sender_to_ui, self.current_block - 1)?,
            UIRequest::PrevBlockInfo => self.handle_get_block_info(node, sender_to_ui, self.current_block + 1)?,
            //UIRequest::MerkleProof(tx) => todo!(),
            UIRequest::EndOfProgram => *program_running = false,
        };
    
        Ok(self)
    }

    fn handle_get_block_info(&mut self, node: &Node, sender_to_ui: &GlibSender<UIResponse>, block_number: usize) -> Result<(), WalletError>{
        let block_info = match node.get_block_info_from(block_number){
            Ok(block_info) => block_info,
            Err(error) => {
                let wallet_error = match error {
                    NodeError::ErrorFindingBlock => WalletError::ErrorFindingBlock,   
                    _ => WalletError::ErrorGettingBlockInfo,
                };
                return sender_to_ui.send(UIResponse::WalletError(wallet_error)).map_err(|_| WalletError::ErrorSendingToUI);
            }
        };
        
        self.current_block = block_info.block_number;
        
        sender_to_ui.send(UIResponse::BlockInfo(block_info)).map_err(|_| WalletError::ErrorSendingToUI)?; 
        
        Ok(())
    }
    
    fn handle_last_block_info(&mut self, node: &Node, sender_to_ui: &GlibSender<UIResponse>) -> Result<(), WalletError>{
        let block_info = match node.get_last_block_info(){
            Ok(block_info) => block_info,
            Err(error) => {
                let wallet_error = match error {
                    NodeError::ErrorFindingBlock => WalletError::ErrorFindingBlock,   
                    _ => WalletError::ErrorGettingBlockInfo,
                };
                return sender_to_ui.send(UIResponse::WalletError(wallet_error)).map_err(|_| WalletError::ErrorSendingToUI);
            }
        };
        
        self.current_block = block_info.block_number;
        
        sender_to_ui.send(UIResponse::BlockInfo(block_info)).map_err(|_| WalletError::ErrorSendingToUI)?; 
        
        Ok(())
    }

    fn handle_update(&self, sender_to_ui: &GlibSender<UIResponse>) -> Result<(), WalletError>{
        let wallet_info = WalletInfo::from(self);
        
        sender_to_ui.send(UIResponse::WalletInfo(wallet_info)).map_err(|_| WalletError::ErrorSendingToUI)?;

        Ok(())
    }

    fn handle_change_wallet(&self, node: &mut Node, priv_key_string: String, sender_to_ui: &GlibSender<UIResponse>) -> Result<Wallet, WalletError>{
        let mut new_wallet = Wallet::from(priv_key_string)?;
        node.set_wallet(&mut new_wallet).map_err(|_| WalletError::ErrorSetingWallet)?;
        new_wallet.handle_update(sender_to_ui)?;
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