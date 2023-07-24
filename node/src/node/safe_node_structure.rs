use crate::{
    blocks::{blockchain::*, Transaction},
    utils::NodeError,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

pub type SafeBlockChain = Arc<Mutex<HashMap<[u8; 32], Block>>>;
pub type SafeVecHeader = Arc<Mutex<Vec<BlockHeader>>>;
pub type SafePendingTx = Arc<Mutex<HashMap<[u8; 32], Transaction>>>;
pub type SafeHeadersIndex = Arc<Mutex<HashMap<[u8; 32], usize>>>;

#[derive(Clone, Debug)]
pub struct NodeSharedInformation {
    pub safe_blockchain: SafeBlockChain,
    pub safe_block_headers: SafeVecHeader,
    pub safe_headers_index: SafeHeadersIndex,
    pub safe_pending_tx: SafePendingTx,
}

impl NodeSharedInformation {
    pub fn from(
        safe_blockchain: &SafeBlockChain,
        safe_block_headers: &SafeVecHeader,
        safe_headers_index: &SafeHeadersIndex,
        safe_pending_tx: &SafePendingTx,
    ) -> NodeSharedInformation {
        NodeSharedInformation {
            safe_blockchain: safe_blockchain.clone(),
            safe_block_headers: safe_block_headers.clone(),
            safe_headers_index: safe_headers_index.clone(),
            safe_pending_tx: safe_pending_tx.clone(),
        }
    }

    pub fn lock_blockchain(&self) -> Result<MutexGuard<HashMap<[u8; 32], Block>>, NodeError> {
        self.safe_blockchain
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }

    pub fn lock_block_headers(&self) -> Result<MutexGuard<Vec<BlockHeader>>, NodeError> {
        self.safe_block_headers
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }

    pub fn lock_headers_index(&self) -> Result<MutexGuard<HashMap<[u8; 32], usize>>, NodeError> {
        self.safe_headers_index
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }

    pub fn lock_safe_pending_tx(
        &self,
    ) -> Result<MutexGuard<HashMap<[u8; 32], Transaction>>, NodeError> {
        self.safe_pending_tx
            .lock()
            .map_err(|_| NodeError::ErrorSharingReference)
    }
}
