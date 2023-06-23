use std::str::FromStr;

use bitcoin_hashes::{hash160, Hash};
use crate::{blocks::transaction::*, utils::WalletError};
use crate::node::*;
use crate::utils::btc_errors::NodeError;
use secp256k1::{SecretKey, PublicKey, Secp256k1};

const BASE_58_CHAR_PRIV_KEY_LENGTH: usize = 52;
const HEX_CHAR_PRIV_KEY_LENGTH: usize = 64;

pub struct Wallet{
    pub_key: PublicKey,
    priv_key: SecretKey,
    balance: i64,
    pending_balance: i64,
}



impl Wallet{
    pub fn new(pub_key: PublicKey, priv_key: SecretKey) -> Wallet{
        
        Wallet {
            pub_key,
            priv_key,
            balance: 0,
            pending_balance: 0,
        }
    }

    pub fn get_pk_hash(&self) -> [u8; 20]{
        hash160::Hash::hash(&self.pub_key.serialize()).to_byte_array()
    }

    pub fn update(&mut self, balance: i64){
        self.balance = balance;
    }

    pub fn from(priv_key_string: String)-> Result<Wallet, WalletError>{
        let mut priv_key: Vec<u8> = Vec::new();
        if priv_key_string.len() == BASE_58_CHAR_PRIV_KEY_LENGTH{
            priv_key = bs58::decode(priv_key_string).into_vec().map_err(|_| WalletError::ErrorHandlingPrivKey)?;;
            priv_key.remove(0);
            priv_key.truncate(priv_key.len()-5);
        };
        //p handelear hexa
        if priv_key.is_empty(){
            return Err(WalletError::ErrorHandlingPrivKey);
        }
        
        let priv_key = SecretKey::from_slice(&priv_key).map_err(|_| WalletError::ErrorHandlingPrivKey)?;;
        
        let pub_key = priv_key.public_key(&Secp256k1::new());
        Ok(Wallet::new(pub_key, priv_key))
    }

    pub fn create_transaction(&self, node: &mut Node, amount: i64, fee: i64, address: [u8; 25]) -> Result<(), NodeError>{
        
        let (unspent_outpoints, unspent_balance) = node.get_utxos_sum_up_to(amount + fee)?;
        //if unspent_outpoints.len() < 2{
        //    panic!("not happening");
        //}
        println!("Se agarraron {} outpoints", unspent_outpoints.len());

        let transaction = Transaction::create(amount, fee, unspent_outpoints, unspent_balance, self.pub_key, self.priv_key, address)
            .map_err(|_| NodeError::ErrorSendingTransaction)?;
        
        node.logger.log(format!("se empezo a enviar la transaccion"));
        node.send_transaction(transaction)?;
        
        Ok(())
    }
}

mod test{

    use super::*;

    #[test]
    fn header_message_test_1_to_bytes_empty_header_message(){
        let priv_key = [
        0x8A, 0x39, 0x27, 0x84, 0x29, 0x20, 0x92, 0xB1, 0x94, 0x1F, 0x8A, 0x72, 0xB0, 0x94, 0x37, 0x16 , 0x04, 0x51, 0x8F, 0x55, 0x30, 0xA5, 0x8D, 0x66, 0xCA, 0x9D, 0xE3, 0x7E, 0x35, 0x6F, 0x8B, 0xBB
        ];
        println!("{:?}", priv_key);
        let priv_key_string = "cSDPYr9FfseHx8jbjrnz9ryERswMkv6vKSccomu1ShfrJXj2d65Z";
        let mut priv_key: Vec<u8> = Vec::new();
        if priv_key_string.len() == BASE_58_CHAR_PRIV_KEY_LENGTH{
            priv_key = bs58::decode(priv_key_string).into_vec().unwrap();
            priv_key.remove(0);
            priv_key.truncate(priv_key.len()-5);
        };
        println!("{:?}", priv_key);
        //p handelear hexa
        if priv_key.is_empty(){
            return
        }
        
        let priv_key = SecretKey::from_slice(&priv_key).unwrap();// from_slice(&priv_key).unwrap();
        
        let pub_key = priv_key.public_key(&Secp256k1::new());
    }
}