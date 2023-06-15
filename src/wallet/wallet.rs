

pub struct wallet{
    pub_key: [u8; 33],
    priv_key: [u8; 64],
    balance: i64,
    pending: i64,
    node: Node,
}

impl wallet{
    fn new(pub_key: [u8;32], priv_key: [u8;64], node: Node){

        let balance = node.get_utxo_balance(pub_key);
        let pending = 0;

        wallet {
            pub_key,
            priv_key,
            balance,
            pending,
            node,
        }
        
    }

    fn create_transaction(){

    }
}