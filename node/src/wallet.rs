pub mod handle_ui_requests;

use crate::node::Node;
use crate::utils::ui_communication_protocol::TxInfo;
use crate::{blocks::transaction::*, utils::WalletError};
use bitcoin_hashes::{hash160, Hash};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::collections::HashMap;

const BASE_58_CHAR_PRIV_KEY_LENGTH: usize = 52;
const HEX_CHAR_PRIV_KEY_LENGTH: usize = 64;

pub struct Wallet {
    pub pub_key: PublicKey,
    priv_key: SecretKey,
    pub balance: i64,
    pub receiving_pending_balance: i64,
    pub sending_pending_balance: i64,
    pub pending_tx: Vec<TxInfo>,
    pub utxos: HashMap<Outpoint, i64>,
    current_block: usize,
}

impl Wallet {
    /// It creates and returns a wallet with the values passed as parameters.
    pub fn new(pub_key: PublicKey, priv_key: SecretKey) -> Wallet {
        Wallet {
            pub_key,
            priv_key,
            balance: 0,
            receiving_pending_balance: 0,
            sending_pending_balance: 0,
            pending_tx: Vec::new(),
            utxos: HashMap::new(),
            current_block: 0,
        }
    }

    pub fn get_pk_hash(&self) -> [u8; 20] {
        hash160::Hash::hash(&self.pub_key.serialize()).to_byte_array()
    }

    /// Creates a wallet interpreting a string as a priv_key written in b58 or hex.
    pub fn from(priv_key_string: String) -> Result<Wallet, WalletError> {
        let priv_key = match priv_key_string.len() {
            BASE_58_CHAR_PRIV_KEY_LENGTH => {
                let mut bytes = bs58::decode(priv_key_string)
                    .into_vec()
                    .map_err(|_| WalletError::ErrorHandlingPrivKey)?;
                bytes.remove(0);
                bytes.truncate(bytes.len() - 5);
                bytes
            }
            HEX_CHAR_PRIV_KEY_LENGTH => {
                match get_bytes_from_hex(priv_key_string) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(WalletError::ErrorHandlingPrivKey),
                }
            }
            _ => return Err(WalletError::ErrorHandlingPrivKey),
        };

        let priv_key =
            SecretKey::from_slice(&priv_key).map_err(|_| WalletError::ErrorHandlingPrivKey)?;

        let pub_key = priv_key.public_key(&Secp256k1::new());
        Ok(Wallet::new(pub_key, priv_key))
    }

    /// Creates a transaction and asks the node to send it
    pub fn create_transaction(
        &mut self,
        node: &mut Node,
        amount: i64,
        fee: i64,
        address: [u8; 25],
    ) -> Result<(), WalletError> {
        let (unspent_outpoints, unspent_balance) = node
            .get_utxos_sum_up_to(amount + fee)
            .map_err(|_| WalletError::ErrorNotEnoughSatoshis)?;
        let transaction = Transaction::create(
            amount,
            fee,
            unspent_outpoints,
            unspent_balance,
            self.pub_key,
            self.priv_key,
            address,
        )
        .map_err(|_| WalletError::ErrorCreatingTx)?;

        node.logger
            .log("se empezo a enviar la transaccion".to_string());
        node.send_transaction(self, transaction)
            .map_err(|_| WalletError::ErrorSendingTx)?;

        Ok(())
    }

    /// Updates the wallet information regarding unspent transactions
    pub fn update_pending_tx(&mut self, pending_tx_info: Vec<TxInfo>) {
        let mut new_pending_tx_info = Vec::new();
        self.receiving_pending_balance = 0;
        self.sending_pending_balance = 0;

        for mut node_pending in pending_tx_info {
            for wallet_pending in &self.pending_tx {
                if wallet_pending.hash == node_pending.hash {
                    node_pending.tx_out_total = wallet_pending.tx_out_total;
                    node_pending.tx_in_total = wallet_pending.tx_in_total;
                    break;
                }
            }
            if node_pending.tx_in_total == 0 {
                self.sending_pending_balance += 0;
            } else {
                self.sending_pending_balance +=
                    node_pending.tx_in_total + node_pending.tx_out_total;
            }
            self.receiving_pending_balance += node_pending.tx_out_total;

            new_pending_tx_info.push(node_pending);
        }
        self.pending_tx = new_pending_tx_info;
    }
}

/// Returns a vec of u8, interpreting the characters of the string as hex.
pub fn get_bytes_from_hex(hex_string: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for chunk in hex_string.as_bytes().chunks(2) {
        let str_chunk = std::str::from_utf8(chunk)?;
        let value = u8::from_str_radix(str_chunk, 16)?;
        result.push(value);
    }
    Ok(result)
}
