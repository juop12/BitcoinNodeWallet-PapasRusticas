use std::fmt::Debug;


pub trait BtcError: Debug{
    fn to_string(&self)-> String{
        format!("Error: {:?}", self)
    }
}

#[derive(Debug)]
pub enum BlockChainError {
    ErrorCreatingBlock,
    ErrorSendingBlock,
    ErrorCreatingBlockHeader,
    ErrorSendingBlockHeader,
}

impl BtcError for BlockChainError {}

/// Enum that represents the possible errors that can occur while creating a transaction
#[derive(Debug, PartialEq)]
pub enum TransactionError {
    ErrorCreatingTransaction,
    ErrorCreatingTxInFromBytes,
    ErrorCreatingTxOutFromBytes,
    ErrorCreatingOutpointFromBytes,
}

impl BtcError for TransactionError {}

/// Error Struct for messages, contains customized errors for each type of message (excluding
/// VerACKMessage) and to diferenciate whether the error occured while instanciation or in
/// message sending
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
    ErrorSendingPongMessages,
    ErrorCreatingNotFoundMessage,
    ErrorsendingNotFoundMessage
}

impl BtcError for MessageError {}

#[derive(Debug)]
/// Enum that contains the possible errors that can occur when running the block downloader.
pub enum BlockDownloaderError {
    ErrorInvalidCreationSize,
    ErrorSendingToThread,
    ErrorReceivingBlockMessage,
    ErrorSendingMessageBlockDownloader,
    ErrorCreatingWorker,
    ErrorWrokerPaniced,
    ErrorValidatingBlock,
    ErrorReceivingNotFoundMessage,
    BundleNotFound,
    ErrorAllWorkersFailed,
}

impl BtcError for BlockDownloaderError {}

/// Enum that represents the errors that can occur in the NodeDataHandler
#[derive(Debug)]
pub enum NodeDataHandlerError {
    ErrorCreatingNodeDataHandler,
    ErrorOpeningFile,
    ErrorWritingInFile,
    ErrorFlushingWriter,
    ErrorReadingHeaders,
    ErrorReadingBlocks,
    ErrorReadingBytes,
}

impl BtcError for NodeDataHandlerError {}

/// Struct that represents errors that can occur with the config setup.
#[derive(Debug)]
pub enum ConfigError{
    ErrorReadingFile,
    ErrorFillingAttributes,
    ErrorMismatchedFileName,
    ErrorMismatchedQuantityOfParameters,
    ErrorMismatchedParameters,
    ErrorParsingDate,
}

impl BtcError for ConfigError {}

/// Struct that represents the errors that can occur in the Node
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
}

impl BtcError for NodeError {}