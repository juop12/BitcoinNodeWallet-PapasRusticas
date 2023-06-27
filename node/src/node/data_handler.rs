use crate::{node::*, utils::btc_errors::NodeDataHandlerError};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
};

const BLOCKHEADER_SIZE: usize = 80;

/// Struct that handles the data persistance of the node.  It has two readers and two writers, one for each file.
/// The headers reader and writer are used to read and write the headers file, and the blocks
/// reader and writer are used to read and write the blocks file.
#[derive(Debug)]
pub struct NodeDataHandler {
    headers_reader: BufReader<File>,
    blocks_reader: BufReader<File>,
    headers_writer: BufWriter<File>,
    blocks_writer: BufWriter<File>,
}

/// Opens the file in the given path with the given permissions of reading and appending
/// passed by parameter. On error returns NodeDataHandlerError
fn open_file(file_path: &str, read: bool, append: bool) -> Result<File, NodeDataHandlerError> {
    let opened_file = OpenOptions::new()
        .read(read)
        .write(true)
        .append(append)
        .create(true)
        .open(file_path);
    match opened_file {
        Ok(file) => Ok(file),
        Err(_) => Err(NodeDataHandlerError::ErrorOpeningFile),
    }
}

/// Receives a writer and flushes it
fn flush_writer(writer: &mut BufWriter<File>) -> Result<(), NodeDataHandlerError> {
    match writer.flush() {
        Ok(_) => Ok(()),
        Err(_) => Err(NodeDataHandlerError::ErrorFlushingWriter),
    }
}

/// Receives a writer and a vector of bytes and writes the bytes in the file
/// It also writes an endline character at the end of the bytes. On error returns NodeDataHandlerError
fn write_to_file(writer: &mut BufWriter<File>, bytes: &[u8]) -> Result<(), NodeDataHandlerError> {
    if writer.write_all(bytes).is_err() {
        return Err(NodeDataHandlerError::ErrorWritingInFile);
    }
    flush_writer(writer)?;
    Ok(())
}

/// Receives a line from the file and returns a vector of bytes parsed from it.
/// On error returns NodeDataHandlerError.
fn get_bytes_from_file(reader: &mut BufReader<File>) -> Result<Vec<u8>, NodeDataHandlerError> {
    let mut bytes: Vec<u8> = Vec::new();
    match reader.read_to_end(&mut bytes) {
        Ok(_) => Ok(bytes),
        Err(_) => Err(NodeDataHandlerError::ErrorReadingBytes),
    }
}

impl NodeDataHandler {
    /// Creates a new NodeDataHandler.
    pub fn new(
        headers_file_path: &str,
        blocks_file_path: &str,
    ) -> Result<NodeDataHandler, NodeDataHandlerError> {
        let read_headers_file = open_file(headers_file_path, true, false)?;
        let write_headers_file = open_file(headers_file_path, false, true)?;
        let read_blocks_file = open_file(blocks_file_path, true, false)?;
        let write_blocks_file = open_file(blocks_file_path, false, true)?;

        let headers_writer = BufWriter::new(write_headers_file);
        let blocks_writer = BufWriter::new(write_blocks_file);
        let headers_reader = BufReader::new(read_headers_file);
        let blocks_reader = BufReader::new(read_blocks_file);

        Ok(NodeDataHandler {
            headers_reader,
            blocks_reader,
            headers_writer,
            blocks_writer,
        })
    }

    pub fn get_all_headers(&mut self) -> Result<Vec<BlockHeader>, NodeDataHandlerError> {
        let mut headers: Vec<BlockHeader> = Vec::new();
        let reader_reference = &mut self.headers_reader;
        let bytes_from_file = get_bytes_from_file(reader_reference)?;
        let mut headers_bytes = bytes_from_file.as_slice();

        while headers_bytes.len() >= BLOCKHEADER_SIZE {
            let curr_header: &[u8];
            (curr_header, headers_bytes) = headers_bytes.split_at(BLOCKHEADER_SIZE);
            match BlockHeader::from_bytes(curr_header) {
                Ok(header) => headers.push(header),
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingHeaders),
            }
        }
        Ok(headers)
    }

    /// Returns a vector with all the blocks stored in the blocks file.
    /// This function is only called once when the node starts, since dealing with
    /// reading and writing on the same file at the same time can produce
    /// unexpected results. On error returns NodeDataHandlerError
    pub fn get_all_blocks(&mut self) -> Result<Vec<Block>, NodeDataHandlerError> {
        let mut blocks: Vec<Block> = Vec::new();
        let reader_reference = &mut self.blocks_reader;
        let bytes_from_file = get_bytes_from_file(reader_reference)?;
        let mut block_bytes = bytes_from_file.as_slice();

        while !block_bytes.is_empty() {
            let block = match Block::from_bytes(block_bytes) {
                Ok(block) => block,
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingBlocks),
            };
            (_, block_bytes) = block_bytes.split_at(block.amount_of_bytes());
            blocks.push(block);
        }
        Ok(blocks)
    }

    /// Saves the block (as bytes) passed by parameter in the blocks file.
    /// On error returns NodeDataHandlerError
    pub fn save_header(&mut self, header: &BlockHeader) -> Result<(), NodeDataHandlerError> {
        let header_bytes = header.to_bytes();
        write_to_file(&mut self.headers_writer, &header_bytes)?;
        Ok(())
    }

    /// Saves the headers starting form start (as bytes) passed by parameter in the headers file.
    /// On error returns NodeDataHandlerError
    pub fn save_headers_to_disk(
        &mut self,
        safe_headers: &SafeVecHeader,
        start: usize,
    ) -> Result<(), NodeDataHandlerError> {
        let block_headers = safe_headers
            .lock()
            .map_err(|_| NodeDataHandlerError::ErrorSharingData)?;

        for header in block_headers.iter().skip(start) {
            self.save_header(header)?;
        }
        Ok(())
    }

    /// Saves the block (as bytes) passed by parameter in the blocks file.
    /// On error returns NodeDataHandlerError
    pub fn save_block(&mut self, block: &Block) -> Result<(), NodeDataHandlerError> {
        let block_bytes = block.to_bytes();
        write_to_file(&mut self.blocks_writer, &block_bytes)?;
        Ok(())
    }

    /// Saves the blocks starting form start (as bytes) passed by parameter in the headers file.
    /// On error returns NodeDataHandlerError
    pub fn save_blocks_to_disk(
        &mut self,
        safe_blockchain: &SafeBlockChain,
        safe_headers: &SafeVecHeader,
        start: usize,
    ) -> Result<(), NodeDataHandlerError> {
        let block_headers = safe_headers
            .lock()
            .map_err(|_| NodeDataHandlerError::ErrorSharingData)?;
        let blockchain = safe_blockchain
            .lock()
            .map_err(|_| NodeDataHandlerError::ErrorSharingData)?;
        for header in block_headers.iter().skip(start) {
            if let Some(block) = blockchain.get(&header.hash()) {
                self.save_block(block)?;
            }
        }

        Ok(())
    }
}
