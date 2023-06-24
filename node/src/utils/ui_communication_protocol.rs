use crate::utils::btc_errors::{NodeError, WalletError};
use crate::blocks::BlockHeader;
use crate::blocks::Outpoint;
use crate::wallet::Wallet;


pub const TX_PAGE_LENGTH: usize = 30;
pub const BLOCK_PAGE_LENGTH: usize = 10;


pub enum UIToWalletCommunication {
    ChangeWallet(/* private key*/String),    //ui se tiene que fijar que las longitudes esten bien, ya sea en hexa o en base 58. La wallet dependiendo de la cantidad lo pasa a array, y cambia la wallet. 
    CreateTx(/* amount*/i64, /* fee*/i64, /*addres */String),         //ui manda en distintas bases el adrress, se fijan las longitudes
    Update,  //la wallet devuelve, un struct, con el balance, unspent balance, pagina actual de tx, ultimos 5 o 10headers
    //ObtainTxProof,
    EndOfProgram,
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
    receiving_pending_balance: i64,
    sending_pending_balance: i64,
    utxos: Vec<UTxOInfo>,
    pending_tx: Vec<TxInfo>
}

impl WalletInfo{
    pub fn from(wallet: &Wallet) -> WalletInfo{
        let utxos = wallet.utxos.iter()
            .map(|(outpoint, amount)| UTxOInfo::new(*outpoint, *amount))
            .collect();
        
        WalletInfo {
            available_balance: wallet.balance,
            receiving_pending_balance: wallet.receiving_pending_balance,
            sending_pending_balance: wallet.sending_pending_balance,
            utxos,
            pending_tx: wallet.pending_tx.clone(),
        }
    }
}

pub struct UTxOInfo{
    outpoint: Outpoint,
    amount: i64,
}

impl UTxOInfo{
    pub fn new(outpoint: Outpoint, amount: i64)-> UTxOInfo{
        UTxOInfo{outpoint, amount}
    }
}

pub struct BlockInfo{
    pub block_number: usize,
    block_header: BlockHeader,
    block_tx_hashes: Vec<[u8;32]>,
}

impl BlockInfo{
    pub fn new(block_number: usize, block_header: BlockHeader, block_tx_hashes: Vec<[u8;32]>) -> BlockInfo{
        BlockInfo {
            block_number,
            block_header,
            block_tx_hashes
        }
    }
}

#[derive(Clone)]
pub struct TxInfo{
    hash: [u8;32],
    pub amount: i64, //positivo o negativo si sale
}

impl TxInfo{
    pub fn new(hash: [u8;32], amount: i64) -> TxInfo{
        TxInfo { 
            hash, 
            amount
        }
    }
}
