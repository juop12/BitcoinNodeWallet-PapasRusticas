use std::fmt::Debug;

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
    ErrorCreatingNotFoundMessage,
    ErrorSendingNotFoundMessage,
    ErrorSendingBlockHeadersMessage,
    ErrorSendingTxMessage,
    ErrorCreatingPingMessage,
    ErrorSendingPingMessage,
    ErrorCreatingPongMessage,
    ErrorSendingPongMessage,
    UnknownMessage
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
    ErrorWorkerPanicked,
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
    ErrorMismatchedFileName,
    ErrorMismatchedQuantityOfParameters,
    ErrorParsingVersion,
    ErrorParsingIP,
    ErrorParsingPort,
    ErrorParsingDate,
    ErrorInvalidDate,
    ErrorParsingIPV6Bool,
    ErrorInvalidParameter,
    ErrorNoExternalAddressGiven,
    ErrorParameterNotFound,
}

impl BtcError for ConfigError {}

/// Struct that represents the errors that can occur in the Node.
#[derive(Debug)]
pub enum NodeError {
    ErrorConnectingToPeer,
    ErrorSendingMessageInHandshake,
    ErrorReceivingMessageInHandshake,
    ErrorInterpretingMessageCommandName,
    ErrorSendingMessageInIBD,
    ErrorIteratingStreams,
    ErrorReceivingHeadersMessageInIBD,
    ErrorReceivingHeadersMessageHeaderInIBD,
    ErrorCreatingBlockDownloader,
    ErrorDownloadingBlockBundle,
    ErrorCreatingNode,
    ErrorSavingDataToDisk,
    ErrorLoadingDataFromDisk,
    ErrorRecevingBroadcastedInventory,
    ErrorReceivingBroadcastedBlock,
    ErrorValidatingBlock,
    ErrorSharingReference,
    ErrorGettingUtxo,
    ErrorGettingTx,
    ErrorSendingTransaction,
    ErrorNotEnoughSatoshis,
    ErrorFindingBlock,
    ErrorSendingThroughChannel,
    ErrorJoiningThread,
    ErrorPeerTimeout,
    ErrorReceivingMessageHeader,
    DoubleHeader,
    ErrorDisconectedFromBlockchain,
    ErrorMessage(MessageError),
}

impl BtcError for NodeError {
    fn to_string(&self) -> String {
        match self{
            NodeError::ErrorMessage(message_error) => message_error.to_string(),
            _ => format!("Error: {:?}", self),
        }
        
    }
}

#[derive(Debug)]
pub enum PeerComunicatorError {
    ErrorReceivingMessages,
    ErrorWorkerPanicked,
    ErrorFinishingReceivingMessages,
    ErrorCreatingWorker,
    ErrorCantReceiveNewPeerConections,
    ErrorSendingMessage,
    ErrorPropagating,
    LostConnectionToManager,
}

impl BtcError for PeerComunicatorError {}

#[derive(Debug)]
pub enum WorkerError{
    ErrorWorkerPanicked,
    ErrorComunicatingBetweenWorkers,
}

impl BtcError for WorkerError {}

#[derive(Debug,PartialEq)]
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
    InvalidAmount,
    ErrorDisconectedFromBlockchain
}

impl BtcError for WalletError {}
