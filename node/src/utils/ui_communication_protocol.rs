use crate::utils::btc_errors::WalletError;
use crate::blocks::BlockHeader;
use crate::blocks::Outpoint;
use crate::wallet::Wallet;


pub const TX_PAGE_LENGTH: usize = 30;
pub const BLOCK_PAGE_LENGTH: usize = 10;

pub enum UIToWalletCommunication {
    ChangeWallet(/* private key*/String),                              //ui se tiene que fijar que las longitudes esten bien, ya sea en hexa o en base 58. La wallet dependiendo de la cantidad lo pasa a array, y cambia la wallet. 
    CreateTx(/* amount*/i64, /* fee*/i64, /*addres */String),         //ui manda en distintas bases el adrress, se fijan las longitudes
    ObtainTxProof(/*txhash */[u8;32], /*block number */ usize),
    EndOfProgram,
    LastBlockInfo,
    NextBlockInfo,
    PrevBlockInfo,
}

pub enum WalletToUICommunication {
    WalletInfo(WalletInfo),
    BlockInfo(BlockInfo),
    WalletError(WalletError),
    ResultOFTXProof(bool),
    FinishedInitializingNode,
    ErrorInitializingNode,
    TxSent,
    WalletFinished,
}

pub struct WalletInfo{
    pub available_balance: i64,
    pub receiving_pending_balance: i64,
    pub sending_pending_balance: i64,
    pub utxos: Vec<UTxOInfo>,
    pub pending_tx: Vec<TxInfo>
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

#[derive(Clone)]
pub struct UTxOInfo{
    pub outpoint: Outpoint,
    pub amount: i64,
}

impl UTxOInfo{
    pub fn new(outpoint: Outpoint, amount: i64)-> UTxOInfo{
        UTxOInfo{outpoint, amount}
    }
}

pub struct BlockInfo{
    pub block_number: usize,
    pub block_header: BlockHeader,
    pub block_tx_hashes: Vec<[u8;32]>,
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

#[derive(Clone, PartialEq)]
pub struct TxInfo{
    pub hash: [u8;32],
    pub tx_in_total: i64,
    pub tx_out_total: i64,
}

impl TxInfo{
    pub fn new(hash: [u8;32], tx_in_total: i64, tx_out_total: i64) -> TxInfo{
        TxInfo { 
            hash, 
            tx_in_total,
            tx_out_total,
        }
    }
}