use crate::utils::btc_errors::{NodeError, ConfigError};
use crate::blocks::BlockHeader;

const TX_PAGE_LENGTH: usize = 30;
const BLOCK_PAGE_LENGTH: usize = 10;

pub enum UIToWalletCommunication {
    ChangeWallet(/* pubkey*/String, /* private key*/String),    //ui se tiene que fijar que las longitudes esten bien, ya sea en hexa o en base 58. La wallet dependiendo de la cantidad lo pasa a array, y cambia la wallet. 
    CreateTx(/* amount*/i64, /* fee*/i64, /*addres */),         //ui manda en distintas bases el adrress, se fijan las longitudes
    Update,  //la wallet devuelve, un struct, con el balance, unspent balance, pagina actual de tx, ultimos 5 o 10headers
    //ObtainTxProof,
    NextTxPage, //p temporalmente como string, despues creamos el tipo de dato correspondiente
    PrevTxPage,
}

pub enum TxType{
    Sent,
    Received,
    Other
}

pub enum WalletToUICommunication {
    WalletInfo(/*struct info */),
    NodeRunningError(NodeError),
    ErrorInitializingNode,
}

pub struct WalletInfo{
    balance: i64,
    unspent_balance: i64,
    tx_page: [TxInfo;TX_PAGE_LENGTH],
    last_headers: [BlockHeader;BLOCK_PAGE_LENGTH],
}

pub struct TxInfo{
    block_header: BlockHeader,
    tx_type: TxType,
    mined: bool,
    amount: i64,
}
