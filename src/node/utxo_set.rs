use crate::node::*;

impl Node {


    pub fn create_utxo_set(&self) -> Result<(), NodeError>{

        let tx_in_vector = Vec::new();

        self.blockchain.for_each(|block| block.for_each(|tx| {
            tx_in_vector.extend(tx.tx_in);
        }));

    }
}