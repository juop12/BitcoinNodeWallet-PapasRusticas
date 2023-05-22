use crate::{
    node::*,
    utils::btc_errors::NodeDataHandlerError,
};
use std::{
    fs::{File, OpenOptions}, 
    io::{BufWriter,BufReader, BufRead},
};


const HEADERS_FILE_PATH: &str = "data/headers.csv";
const BLOCK_FILE_PATH: &str = "data/blocks.csv";


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
fn open_file(file_path: &str, read: bool, append: bool)-> Result<File, NodeDataHandlerError>{
    let opened_file = OpenOptions::new()
        .read(read)
        .write(true)
        .append(append)
        .create(true)
        .open(file_path);
    match opened_file{
        Ok(file) => Ok(file),
        Err(_) => return Err(NodeDataHandlerError::ErrorOpeningFile),
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
    let mut aux = "";
    for byte in bytes {
        match writer.write_all(format!("{}{}",aux, byte).as_bytes()){
            Ok(_) => (),
            Err(_) => return Err(NodeDataHandlerError::ErrorWritingInFile),
        }
        aux =",";
    }
    flush_writer(writer)?;
    match writer.write_all(b"\n"){
        Ok(_) => (),
        Err(_) => return Err(NodeDataHandlerError::ErrorWritingInFile),
    };
    flush_writer(writer)?;
    Ok(())
}

/// -
fn get_bytes_from_line(line: String)->Result<Vec<u8>, NodeDataHandlerError>{
    let mut bytes: Vec<u8> = Vec::new();
    for byte_str in line.split(",") {
        match byte_str.parse::<u8>(){
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(NodeDataHandlerError::ErrorReadingBytes),
        }
    }
    Ok(bytes)
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

        Ok(NodeDataHandler{
            headers_reader,
            blocks_reader,
            headers_writer,
            blocks_writer
        })
    }

    /// Gets all the headers stored on the headers file. This function is only called
    /// once when the node starts, since dealing with
    /// reading and writing on the same file at the same time can produce
    /// unexpected results. On error returns NodeDataHandlerError
    pub fn get_all_headers(&mut self) -> Result<Vec<BlockHeader>, NodeDataHandlerError> {
        let mut headers :Vec<BlockHeader> = Vec::new();
        let reader_reference = &mut self.headers_reader;
        for line_to_read in reader_reference.lines(){
            let line = match line_to_read{
                Ok(line) => line,
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingHeaders),
            };
            let mut header_bytes = get_bytes_from_line(line)?;
           
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
    pub fn get_all_blocks(&mut self) -> Result<Vec<Block>, NodeDataHandlerError> {
        let mut blocks :Vec<Block> = Vec::new();
        let reader_reference = &mut self.blocks_reader;
        for line_to_read in reader_reference.lines() {
            let line = match line_to_read{
                Ok(line) => line,
                Err(_) => return Err(NodeDataHandlerError::ErrorReadingBlocks),
            };
            let mut block_bytes = get_bytes_from_line(line)?;
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
    pub fn save_header(&mut self, header: &BlockHeader) -> Result<(), NodeDataHandlerError>{
        let header_bytes = header.to_bytes();
        write_to_file(&mut self.headers_writer, &header_bytes)?;
        Ok(())
    }
    
    /// Saves the block (as bytes) passed by parameter in the blocks file.
    /// On error returns NodeDataHandlerError
    pub fn save_block(&mut self, block: &Block) -> Result<(), NodeDataHandlerError>{
        let block_bytes = block.to_bytes();
        write_to_file(&mut self.blocks_writer, &block_bytes)?;
        Ok(())
    }

    pub fn save_to_disk(&mut self, blocks: &HashMap<[u8; 32], Block>, headers: &Vec<BlockHeader>, start: usize) -> Result<(), NodeDataHandlerError>{
        for i in start..headers.len(){
            let header = &headers[i];
            self.save_header(header)?;
            if let Some(block) = blocks.get(&header.hash()){
                self.save_block(block)?;
            }
        }
 
        Ok(())
    }
}

/*
/// -
#[cfg(test)]

mod tests{
    use super::*;
    
    
    #[test]
    fn lectura_headers()-> Result<(), NodeDataHandlerError>{
        let hash = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
        
        let mut data_handler = NodeDataHandler::new()?;

        let headers = data_handler.get_all_headers()?;
        assert_eq!(headers.len(), 3);
        for h in headers{
            assert_eq!(h.prev_hash, hash);
        } 
        Ok(())
    }

    #[test]
    fn lectura_blocks()-> Result<(), NodeDataHandlerError>{
        let hash  = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
        
        let mut data_handler = NodeDataHandler::new()?;

        let blocks = data_handler.get_all_blocks()?;
        assert_eq!(blocks.len(), 3);
        for b in blocks{
            assert_eq!(b.header.prev_hash, hash);
        } 
        Ok(())
    }

    #[test]
    fn escritura()-> Result<(), NodeDataHandlerError>{
        let hash = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
        let header1 = BlockHeader::new(70015, hash,hash);
        let header2 = BlockHeader::new(70015, hash,hash);
        let header3 = BlockHeader::new(70015, hash,hash);
         
        let mut data_handler = NodeDataHandler::new()?;
        data_handler.save_header(&header1)?;
        data_handler.save_header(&header2)?;
        data_handler.save_header(&header3)?;
        Ok(())
    }

    #[test]
    fn escritura_blocks()-> Result<(), NodeDataHandlerError>{
        let hash = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
        let block = Block{
            header: BlockHeader::new(70015, hash,hash),
            transaction_count: vec![1,2,3,4],
            transactions: vec![5,6,7,8],
        };

        let block2 = Block{
            header: BlockHeader::new(70015, hash,hash),
            transaction_count: vec![1,2,3,4],
            transactions: vec![5,6,7,8],
        };

        let block3 = Block{
            header: BlockHeader::new(70015, hash,hash),
            transaction_count: vec![1,2,3,4],
            transactions: vec![5,6,7,8],
        };

        let mut data_handler = NodeDataHandler::new()?;
        data_handler.save_block(&block)?;
        data_handler.save_block(&block2)?;
        data_handler.save_block(&block3)?;
        Ok(())
    }
}*/
