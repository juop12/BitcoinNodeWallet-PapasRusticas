use super::Wallet;
use crate::node::Node;
use crate::utils::ui_communication_protocol::{
    UIRequest, WalletInfo, UIResponse,
};
use crate::utils::NodeError;
use crate::utils::WalletError;
use glib::Sender as GlibSender;

impl Wallet {
    /// Main fucntions, that calls to the correspoding handle, depending on what the ui requested.
    /// If the individual handle fails the it sends the error to the ui but this function does not
    /// return error. The only circumstance when this function returns error is if it cannot
    /// comunicate with th ui
    pub fn handle_ui_request(
        mut self,
        node: &mut Node,
        request: UIRequest,
        sender_to_ui: &GlibSender<UIResponse>,
        program_running: &mut bool,
    ) -> Result<Wallet, WalletError> {
        let ui_response = match request {
            UIRequest::ChangeWallet(priv_key_string) => {
                match self.handle_change_wallet(node, priv_key_string) {
                    Ok(wallet) => return Ok(wallet),
                    Err(wallet_error) => Err(wallet_error),
                }
            }
            UIRequest::CreateTx(amount, fee, address) => {
                self.handle_create_tx(node, amount, fee, address)
            }
            UIRequest::UpdateWallet => self.handle_update_wallet(node),
            UIRequest::LastBlockInfo => self.handle_last_block_info(node),
            UIRequest::NextBlockInfo => self.handle_get_block_info(node, self.current_block - 1),
            UIRequest::PrevBlockInfo => self.handle_get_block_info(node, self.current_block + 1),
            UIRequest::ObtainTxProof(hash, block_index) => {
                self.handle_obtain_tx_proof(node, hash, block_index)
            }
            UIRequest::EndOfProgram => {
                *program_running = false;
                return Ok(self);
            }
        };

        match ui_response {
            Ok(ui_response) => sender_to_ui
                .send(ui_response)
                .map_err(|_| WalletError::ErrorSendingToUI)?,
            Err(wallet_error) => sender_to_ui
                .send(UIResponse::WalletError(wallet_error))
                .map_err(|_| WalletError::ErrorSendingToUI)?,
        }
        Ok(self)
    }

    /// Updates the Wallet information and then returns it in a WalletInfo.
    /// If theres a problem with obtaining the SafeVectors, then it
    /// returns ErrorUpdatingWallet.
    pub fn handle_update_wallet(&mut self, node: &mut Node) -> Result<UIResponse, WalletError> {
        node.update(self)
            .map_err(|_| WalletError::ErrorUpdatingWallet)?;

        let wallet_info = WalletInfo::from(self);

        Ok(UIResponse::WalletInfo(wallet_info))
    }

    /// Returns the block info of the requested block number inside a UiResponse.
    /// If the block was not found it returns error_finding block, on any other
    /// error return ErrorGettingBlockInfo
    pub fn handle_get_block_info(
        &mut self,
        node: &Node,
        block_number: usize,
    ) -> Result<UIResponse, WalletError> {
        let block_info = match node.get_block_info_from(block_number) {
            Ok(block_info) => block_info,
            Err(error) => {
                let wallet_error = match error {
                    NodeError::ErrorFindingBlock => WalletError::ErrorFindingBlock,
                    _ => WalletError::ErrorGettingBlockInfo,
                };
                return Err(wallet_error);
            }
        };

        self.current_block = block_info.block_number;

        Ok(UIResponse::BlockInfo(block_info))
    }

    /// Returns the block info of the last block of the blockchain inside a UiResponse.
    /// If the block was not found it returns error_finding block, on any other
    /// error return ErrorGettingBlockInfo
    pub fn handle_last_block_info(&mut self, node: &Node) -> Result<UIResponse, WalletError> {
        let block_info = match node.get_last_block_info() {
            Ok(block_info) => block_info,
            Err(error) => {
                let wallet_error = match error {
                    NodeError::ErrorFindingBlock => WalletError::ErrorFindingBlock,
                    _ => WalletError::ErrorGettingBlockInfo,
                };
                return Err(wallet_error);
            }
        };

        self.current_block = block_info.block_number;

        Ok(UIResponse::BlockInfo(block_info))
    }

    /// Changes wallet to the one whose private key is written in the string either in base 58 or hex.
    pub fn handle_change_wallet(
        &self,
        node: &mut Node,
        priv_key_string: String,
    ) -> Result<Wallet, WalletError> {
        let mut new_wallet = Wallet::from(priv_key_string)?;
        node.set_wallet(&mut new_wallet)
            .map_err(|_| WalletError::ErrorSettingWallet)?;

        Ok(new_wallet)
    }

    /// Creates and sends a transaction to the receiver address of value amount and fee.
    fn handle_create_tx(
        &mut self,
        node: &mut Node,
        amount: i64,
        fee: i64,
        receiver_address: String,
    ) -> Result<UIResponse, WalletError> {
        let address_bytes = bs58::decode(receiver_address)
            .into_vec()
            .map_err(|_| WalletError::ErrorSendingTx)?;
        let mut address: [u8; 25] = [0; 25];
        address.copy_from_slice(&address_bytes);
        self.create_transaction(node, amount, fee, address)?;

        Ok(UIResponse::TxSent)
    }

    /// Requests the merkle proof of inclution to the node and verifies it.
    pub fn handle_obtain_tx_proof(
        &self,
        node: &Node,
        tx_hash: [u8; 32],
        block_index: usize,
    ) -> Result<UIResponse, WalletError> {
        let (merkle_proof, merkle_root) = node
            .get_merkle_tx_proof(tx_hash, block_index)
            .map_err(|_| WalletError::ErrorObtainingTxProof)?;

        let mut prev_hash = tx_hash;

        for hash_pair in &merkle_proof {
            if hash_pair.contains(prev_hash) {
                prev_hash = hash_pair.hash();
            } else {
                return Ok(UIResponse::ResultOFTXProof(None));
            }
        }

        if prev_hash != merkle_root {
            return Ok(UIResponse::ResultOFTXProof(None));
        }

        Ok(UIResponse::ResultOFTXProof(Some((merkle_proof, merkle_root))))
    }
}
