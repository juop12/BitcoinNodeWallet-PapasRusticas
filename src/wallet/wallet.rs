pub struct wallet{
    pub_key: [u8; 33],
    priv_key: [u8; 64],
    balance: i64,
    pending_balance: i64,
    node: Node,
}

impl wallet{
    fn new(pub_key: [u8;32], priv_key: [u8;64], node: Node){

        //let balance = node.get_utxo_balance(pub_key);
        //let pending_balance = node.get_pending_balance(pub_key);

        wallet {
            pub_key,
            priv_key,
            balance: 0,
            pending_balance: 0,
            node,
        }
    }

    pub fn get_pk_hash(&self) -> &[u8]{
        hash160::Hash::hash(&pub_key.as_slice())
    }

    fn create_transaction(){

    }
}