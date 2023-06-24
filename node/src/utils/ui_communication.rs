use crate::utils::btc_errors::{NodeError, WalletError,ConfigError};
use crate::blocks::BlockHeader;
use crate::blocks::Outpoint;

pub const TX_PAGE_LENGTH: usize = 30;
pub const BLOCK_PAGE_LENGTH: usize = 10;

pub enum UIToWalletCommunication {
    ChangeWallet(/* private key*/String),    //ui se tiene que fijar que las longitudes esten bien, ya sea en hexa o en base 58. La wallet dependiendo de la cantidad lo pasa a array, y cambia la wallet. 
    CreateTx(/* amount*/i64, /* fee*/i64, /*addres */String),         //ui manda en distintas bases el adrress, se fijan las longitudes
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
    ConfigError(ConfigError),
    WalletError(WalletError),
}

pub struct WalletInfo{
    pub available_balance: i64,
    pub pending_balance: i64,
    pub utxos: Vec<UTxOInfo>,
    pub pending_tx: Vec<TxInfo>
}

#[derive(Clone)]
pub struct UTxOInfo{
    pub outpoint: Outpoint,
    pub amount: i64,
}

pub struct BlockInfo{
    pub block_number: usize,
    pub block_tx_hash: Vec<[u8; 32]>,
    pub block_header: BlockHeader,
}

pub struct TxInfo{
    pub hash: [u8;32],
    pub amount: i64, //positivo o negativo si sale
}
