use bitcoin_hashes::{hash160, Hash};


pub struct Wallet{
    pub_key: [u8; 33],
    priv_key: [u8; 32],
    balance: i64,
    pending_balance: i64,
}

impl Wallet{
    pub fn new(pub_key: [u8; 33], priv_key: [u8; 32]) -> Wallet{

        //let balance = node.get_utxo_balance(pub_key);
        //let pending_balance = node.get_pending_balance(pub_key);

        Wallet {
            pub_key,
            priv_key,
            balance: 0,
            pending_balance: 0,
        }
    }

    pub fn get_pk_hash(&self) -> [u8; 20]{
        hash160::Hash::hash(self.pub_key.as_slice()).to_byte_array()
    }

    pub fn update(&mut self, balance: i64){
        self.balance = balance;
    }

    fn create_transaction(){

    }
}