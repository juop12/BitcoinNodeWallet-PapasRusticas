use std::{
    fs::{File, OpenOptions}, 
    io::{BufWriter,BufReader, BufRead},
};
use crate::node::*;

const HEADERS_FILE_PATH: &str = "src/node/data/headers.csv";
const BLOCK_FILE_PATH: &str = "src/node/data/blocks.csv";

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

/// Enum that represents the errors that can occur in the NodeDataHandler
#[derive(Debug)]
pub enum NodeDataHandlerError {
    ErrorCreatingNodeDataHandler,
    ErrorOpeningFile,
    ErrorWritingInFile,
    ErrorFlushingWriter,
    ErrorReadingHeaders,
    ErrorReadingBlocks,
}

/// Opens the file in the given path with the given permissions of reading and appending
/// passed by parameter. On error returns NodeDataHandlerError
fn open_file(file_path: &str, read: bool, append: bool)-> Result<File, NodeDataHandlerError>{
    let opened_file = OpenOptions::new()
        .read(read)
        .write(true)
        .append(append)
        .create(true)
        .open(file_path);
    match opened_file{
        Ok(file) => Ok(file),
        Err(e) => {
            println!("{:?}", e);
            return Err(NodeDataHandlerError::ErrorOpeningFile);}
    }
}

/// Receives a writer and flushes it
fn flush_writer(writer: &mut BufWriter<File>) -> Result<(), NodeDataHandlerError>{
    match writer.flush(){
        Ok(_) => Ok(()),
        Err(_) => Err(NodeDataHandlerError::ErrorFlushingWriter),
    }
}

/// Receives a writer and a vector of bytes and writes the bytes in the file
/// It also writes an endline character at the end of the bytes. On error returns NodeDataHandlerError
fn write_to_file(writer: &mut BufWriter<File>, bytes: &[u8]) -> Result<(), NodeDataHandlerError>{
    for byte in bytes {
        match writer.write_all(format!("{},", byte).as_bytes()){
            Ok(_) => (),
            Err(_) => return Err(NodeDataHandlerError::ErrorWritingInFile),
        }
    }
    flush_writer(writer)?;
    match writer.write_all(b"\n"){
        Ok(_) => (),
        Err(_) => return Err(NodeDataHandlerError::ErrorWritingInFile),
    };
    flush_writer(writer)?;
    Ok(())
}

impl NodeDataHandler{
    /// Creates a new NodeDataHandler. 
    pub fn new() -> Result<NodeDataHandler, NodeDataHandlerError>{
    
    let read_headers_file = open_file(HEADERS_FILE_PATH, true, false)?;
    let write_headers_file = open_file(HEADERS_FILE_PATH, false, true)?;
    let read_blocks_file = open_file(BLOCK_FILE_PATH, true, false)?;
    let write_blocks_file = open_file(BLOCK_FILE_PATH, false, true)?;

    let headers_writer = BufWriter::new(write_headers_file);
    let blocks_writer = BufWriter::new(write_blocks_file);
    let headers_reader = BufReader::new(read_headers_file);
    let blocks_reader = BufReader::new(read_blocks_file);

    Ok( NodeDataHandler{
        headers_reader,
        blocks_reader,
        headers_writer,
        blocks_writer}
        )
    }

    /// Gets all the headers stored on the headers file. This function is only called
    /// once when the node starts, since dealing with
    /// reading and writing on the same file at the same time can produce
    /// unexpected results. On error returns NodeDataHandlerError
    pub fn get_all_headers(self) -> Result<Vec<BlockHeader>, NodeDataHandlerError> {
        let mut headers :Vec<BlockHeader> = Vec::new();
        for line_to_read in self.headers_reader.lines() {
            let mut header_bytes = match line_to_read{
                Ok(line) => Vec::from(line.as_bytes()),
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingHeaders),
            };
            let read_header = match BlockHeader::from_bytes(&mut header_bytes){
                Ok(header) => header,
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingHeaders),
            };
            headers.push(read_header);
        }
        Ok(headers)
    }
    /// Returns a vector with all the blocks stored in the blocks file.
    /// This function is only called once when the node starts, since dealing with
    /// reading and writing on the same file at the same time can produce
    /// unexpected results. On error returns NodeDataHandlerError
    pub fn get_all_blocks(self) -> Result<Vec<Block>, NodeDataHandlerError> {
        let mut blocks :Vec<Block> = Vec::new();
        for line_to_read in self.blocks_reader.lines() {
            let mut block_bytes = match line_to_read{
                Ok(line) => Vec::from(line.as_bytes()),
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingBlocks),
            };
            let read_block = match Block::from_bytes(&mut block_bytes){
                Ok(block) => block,
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingBlocks),
            };
            blocks.push(read_block);
        }
        Ok(blocks)
    }
    /// Saves the header (as bytes) passed by parameter in the headers file.
    /// On error returns NodeDataHandlerError
    pub fn save_header(&mut self, header: BlockHeader) -> Result<(), NodeDataHandlerError>{
        let header_bytes = header.to_bytes();
        write_to_file(&mut self.headers_writer, &header_bytes)?;
        Ok(())
    }

    /// Saves the block (as bytes) passed by parameter in the blocks file.
    /// On error returns NodeDataHandlerError
    pub fn save_block(&mut self, block: Block) -> Result<(), NodeDataHandlerError>{
        let block_bytes = block.to_bytes();
        write_to_file(&mut self.blocks_writer, &block_bytes)?;
        Ok(())
    }
}

#[cfg(test)]

mod tests{
    use super::*;
    
    #[test]
    fn data_persistance_test_1_can_save_block_headers()-> Result<(), NodeDataHandlerError>{
        let header = BlockHeader::new(70015, [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31],[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31]);
        let mut data_handler = NodeDataHandler::new()?;
        data_handler.save_header(header)?;
        Ok(())
    }
}
