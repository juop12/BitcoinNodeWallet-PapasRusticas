use crate::utils::btc_errors::{NodeError, WalletError};
use crate::blocks::BlockHeader;
use crate::blocks::Outpoint;

pub const TX_PAGE_LENGTH: usize = 30;
pub const BLOCK_PAGE_LENGTH: usize = 10;

pub enum UIToWalletCommunication {
    ChangeWallet(/* private key*/String),    //ui se tiene que fijar que las longitudes esten bien, ya sea en hexa o en base 58. La wallet dependiendo de la cantidad lo pasa a array, y cambia la wallet. 
    CreateTx(/* amount*/i64, /* fee*/i64, /*addres */),         //ui manda en distintas bases el adrress, se fijan las longitudes
    Update,  //la wallet devuelve, un struct, con el balance, unspent balance, pagina actual de tx, ultimos 5 o 10headers
    //ObtainTxProof,
    LastBlockInfo,
    NextBlockInfo, //p temporalmente como string, despues creamos el tipo de dato correspondiente
    PrevBlockInfo,
}

pub enum WalletToUICommunication {
    WalletInfo(WalletInfo),
    BlockInfo(BlockInfo),
    TxSent,
    ErrorInitializingNode,
    NodeRunningError(NodeError),
    WalletError(WalletError),
}

pub struct WalletInfo{
    available_balance: i64,
    pending_balance: i64,
    utxos: Vec<UTxOInfo>,
    pending_tx: Vec<TxInfo>
}

pub struct UTxOInfo{
    outpoint: Outpoint,
    amount: i64,
}

pub struct BlockInfo{
    block_number: usize,
    block_tx_hash: Vec<[u8;32]>,
    block_header: BlockHeader,
}

pub struct TxInfo{
    hash: [u8;32],
    amount: i64, //positivo o negativo si sale
}
