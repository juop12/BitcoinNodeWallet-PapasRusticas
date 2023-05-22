use crate::node::*;

impl Node {
    
    ///Handles the headers message by hashing the last received header and asking for more headers.
    pub fn handle_block_headers_message(&mut self, mut msg_bytes :Vec<u8>, sync_node_index: usize)-> Result<(), NodeError>{
        let block_headers_msg = match BlockHeadersMessage::from_bytes(&mut msg_bytes){
            Ok(block_headers_message) => block_headers_message,
            Err(_) => return Err(NodeError::ErrorReceivingHeadersMessageInIBD),
        };
        //println!("Recibimos {:?} headers", block_headers_msg.count);
        let received_block_headers = block_headers_msg.headers;
        //let quantity_received = received_block_headers.len();
        
        self.block_headers.extend(received_block_headers);
        Ok(())
    }

    ///Handles the block message by validating the proof of work and the proof of inclusion and the saving it.
    /// If the block is already in the blockchain, it is not saved.
    pub fn handle_block_message(&mut self, mut msg_bytes: Vec<u8>)->Result<(), NodeError>{
        let block_msg = match BlockMessage::from_bytes(&mut msg_bytes){
            Ok(block_msg) => block_msg,
            Err(_) => return Err(NodeError::ErrorReceivingBroadcastedBlock),
        };

        if self.blockchain.contains_key(&block_msg.block.get_header().hash()){
           return Ok(());
        }

        if validate_proof_of_work(&block_msg.block.get_header()){
            if validate_proof_of_inclusion(&block_msg.block){
                self.add_broadcasted_block(block_msg.block)?;
            }else{
                self.logger.log(String::from("Proof of inclusion failed for a block"));
            }
        }else{
            self.logger.log(String::from("Proof of work failed for a block"));
        }
        
        Ok(())
    }

    ///Handles the inv message by asking for the blocks that are not in the blockchain.
    ///If the block is already in the blockchain, it is not saved.
    pub fn handle_inv_message(&mut self, mut msg_bytes: Vec<u8>, stream_index: usize)-> Result<(),NodeError>{
        let inv_msg = match InvMessage::from_bytes(&mut msg_bytes){
            Ok(msg) => msg,
            Err(_) => return Err(NodeError::ErrorRecevingBroadcastedInventory),
        };
        
        let stream = &mut self.tcp_streams[stream_index];
        
        match get_blocks_from_bundle(inv_msg.get_block_hashes(), stream, &self.logger){
            Ok(blocks) => {
                for block in blocks{
                    if !self.blockchain.contains_key(&block.get_header().hash()){
                        self.add_broadcasted_block(block)?;
                    }
                }
                Ok(())
            },
            Err(_) => Err(NodeError::ErrorDownloadingBlockBundle),
        }
    }
    
    ///Handles the ping message by sending a pong message.
    pub fn handle_ping_message(&self, stream_index: usize, header_message: &HeaderMessage, nonce: Vec<u8>)->Result<(),NodeError>{
        if nonce.len() != 8{
            return Err(NodeError::ErrorReceivingPing)
        }
        let mut stream = &self.tcp_streams[stream_index];
        
        let mut pong_bytes = header_message.to_bytes();
        pong_bytes.extend(nonce);
        pong_bytes[5] = b'o';
        
        if stream.write(&pong_bytes).is_err(){
            return Err(NodeError::ErrorSendingPong)
        }
        Ok(())
    }

    ///Adds a block to the blockchain, its header to the headers vector and saves them both on disk.
    fn add_broadcasted_block(&mut self, block: Block)->Result<(),NodeError>{
        match BlockHeader::from_bytes(&mut block.get_header().to_bytes()){
            Ok(header) => self.block_headers.push(header),
            Err(_) => return Err(NodeError::ErrorReceivingBroadcastedBlock),
        };
        if self.data_handler.save_header(block.get_header()).is_err(){
            return Err(NodeError::ErrorSavingDataToDisk);
        }
        if self.data_handler.save_block(&block).is_err(){
            return Err(NodeError::ErrorSavingDataToDisk);
        }
        self.blockchain.insert(block.get_header().hash(), block);
        Ok(())
    }
}