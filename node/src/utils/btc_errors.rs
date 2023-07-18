use std::fmt::Debug;

use crate::node::peer_comunication::workers::Worker;

/// All errors that can occur in the project must implement this trait.
pub trait BtcError: Debug {
    fn to_string(&self) -> String {
        format!("Error: {:?}", self)
    }
}

/// Enum that represents the possible errors that can occur while creating or sending a block or its header.
#[derive(Debug)]
pub enum BlockChainError {
    ErrorCreatingBlock,
    ErrorSendingBlock,
    ErrorCreatingBlockHeader,
    ErrorSendingBlockHeader,
}

impl BtcError for BlockChainError {}

/// Enum that represents the possible errors that can occur while creating a transaction.
#[derive(Debug, PartialEq)]
pub enum TransactionError {
    ErrorCreatingTransaction,
    ErrorCreatingTxInFromBytes,
    ErrorCreatingTxOutFromBytes,
    ErrorCreatingOutpointFromBytes,
    ErrorCreatingSignature,
}

impl BtcError for TransactionError {}

/// Error Struct for messages, contains customized errors for each type of message (excluding
/// VerACKMessage) and to diferenciate whether the error occured while instanciation or in
/// message sending.
#[derive(Debug, PartialEq)]
pub enum MessageError {
    ErrorCreatingVersionMessage,
    ErrorSendingVersionMessage,
    ErrorCreatingHeaderMessage,
    ErrorSendingHeaderMessage,
    ErrorCreatingVerAckMessage,
    ErrorSendingVerAckMessage,
    ErrorCreatingGetBlockHeadersMessage,
    ErrorSendingGetBlockHeadersMessage,
    ErrorCreatingBlockHeadersMessage,
    ErrorHeadersBlockMessage,
    ErrorCreatingGetDataMessage,
    ErrorSendingGetDataMessage,
    ErrorCreatingInvMessage,
    ErrorSendingInvMessage,
    ErrorCreatingBlockMessage,
    ErrorSendingBlockMessage,
    ErrorSendingPongMessages,
    ErrorCreatingNotFoundMessage,
    ErrorSendingNotFoundMessage,
    ErrorSendingBlockHeadersMessage,
    ErrorSendingTxMessage,
}

impl BtcError for MessageError {}

/// Enum that contains the possible errors that can occur when running the block downloader.
#[derive(Debug)]
pub enum BlockDownloaderError {
    ErrorInvalidCreationSize,
    ErrorSendingToThread,
    ErrorReceivingBlockMessage,
    ErrorSendingMessageBlockDownloader,
    ErrorCreatingWorker,
    ErrorWorkerPaniced,
    ErrorValidatingBlock,
    ErrorReceivingNotFoundMessage,
    BundleNotFound,
    ErrorAllWorkersFailed,
    ErrorCreatingBlockDownloader,
}

impl BtcError for BlockDownloaderError {}

/// Enum that represents the errors that can occur in the NodeDataHandler.
#[derive(Debug)]
pub enum NodeDataHandlerError {
    ErrorCreatingNodeDataHandler,
    ErrorOpeningFile,
    ErrorWritingInFile,
    ErrorFlushingWriter,
    ErrorReadingHeaders,
    ErrorReadingBlocks,
    ErrorReadingBytes,
    ErrorSharingData,
}

impl BtcError for NodeDataHandlerError {}

/// Struct that represents errors that can occur with the config setup.
#[derive(Debug)]
pub enum ConfigError {
    ErrorReadingFile,
    ErrorFillingAttributes,
    ErrorMismatchedFileName,
    ErrorMismatchedQuantityOfParameters,
    ErrorMismatchedParameters,
    ErrorParsingDate,
}

impl BtcError for ConfigError {}

/// Struct that represents the errors that can occur in the Node.
#[derive(Debug)]
pub enum NodeError {
    ErrorConnectingToPeer,
    ErrorSendingMessageInHandshake,
    ErrorReceivingMessageInHandshake,
    ErrorReceivedUnknownMessage,
    ErrorInterpretingMessageCommandName,
    ErrorSendingMessageInIBD,
    ErrorIteratingStreams,
    ErrorReceivingHeadersMessageInIBD,
    ErrorReceivingMessageHeader,
    ErrorReceivingHeadersMessageHeaderInIBD,
    ErrorCreatingBlockDownloader,
    ErrorDownloadingBlockBundle,
    ErrorCreatingNode,
    ErrorSavingDataToDisk,
    ErrorLoadingDataFromDisk,
    ErrorRecevingBroadcastedInventory,
    ErrorReceivingBroadcastedBlock,
    ErrorReceivingPing,
    ErrorSendingPong,
    ErrorReceivingMessage,
    ErrorValidatingBlock,
    ErrorSharingReference,
    ErrorGettingUtxo,
    ErrorGettingTx,
    ErrorSendingTransaction,
    ErrorNotEnoughSatoshis,
    ErrorFindingBlock,
    ErrorPeerTimeout,
}

impl BtcError for NodeError {}

#[derive(Debug)]
pub enum PeerComunicatorError {
    ErrorReceivingMessages,
    ErrorAddingReceivedData,
    ErrorWrokerPaniced,
    ErrorFinishingReceivingMessages,
    ErrorCreatingWorker,
    ErrorCantReceiveNewPeerConections,
    ErrorSendingMessage
}

impl BtcError for PeerComunicatorError {}

#[derive(Debug)]
pub enum WorkerError{
    ErrorWorkerPaniced,
    ErrorComunicatingBetweenWorkers,
    LostConnectionToManager,
}

impl BtcError for WorkerError {}

#[derive(Debug)]
pub enum WalletError {
    ErrorHandlingPrivKey,
    ErrorHandlingAddress,
    ErrorSendingTx,
    ErrorCreatingTx,
    ErrorNotEnoughSatoshis,
    ErrorSendingToUI,
    ErrorSettingWallet,
    ErrorFindingBlock,
    ErrorGettingBlockInfo,
    ErrorObtainingTxProof,
    ErrorReceivingFromUI,
    ErrorUpdatingWallet,
}

impl BtcError for WalletError {}
