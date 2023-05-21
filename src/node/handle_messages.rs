use crate::node::*;

impl Node {
    
    pub fn handle_block_message(&mut self, mut msg_bytes: Vec<u8>)->Result<(), NodeError>{
        let block_msg = match BlockMessage::from_bytes(&mut msg_bytes){
            Ok(block_msg) => block_msg,
            Err(_) => return Err(NodeError::ErrorReceivingBroadcastedBlock),
        };

        if validate_proof_of_work(&block_msg.block.get_header()){
            if validate_proof_of_inclusion(&block_msg.block){
                self.add_broadcasted_block(block_msg.block)?;
            }else{
                println!("\n\nfallos proof of inclusion\n\n");
            }
        }else{
            println!("\n\nfallos proof of work\n\n");
        }
        
        Ok(())
    }

    pub fn handle_inv_message(&mut self, mut msg_bytes: Vec<u8>, stream_index: usize)-> Result<(),NodeError>{
        let inv_msg = match InvMessage::from_bytes(&mut msg_bytes){
            Ok(msg) => msg,
            Err(_) => return Err(NodeError::ErrorRecevingBroadcastedInventory),
        };
        
        let stream = &mut self.tcp_streams[stream_index];
        
        match get_blocks_from_bundle(inv_msg.get_block_hashes(), stream){
            Ok(blocks) => {
                for block in blocks{
                    self.add_broadcasted_block(block)?;
                }
                Ok(())
            },
            Err(_) => Err(NodeError::ErrorDownloadingBlockBundle),
        }
    }
    
    pub fn handle_ping_message(&self, stream_index: usize, header_message: &HeaderMessage, nonce: Vec<u8>){
        if nonce.len() != 8{
            return
        }
        let mut stream = &self.tcp_streams[stream_index];
        
        let mut pong_bytes = header_message.to_bytes();
        pong_bytes.extend(nonce);
        pong_bytes[5] = b'o';
        //p manejar desp
        stream.write(&pong_bytes);
    }
    
    pub fn handle_version_message(&self,stream_index: usize, msg_bytes: Vec<u8>) -> Result<(), NodeError>{
        let stream = &self.tcp_streams[stream_index];

        match VersionMessage::from_bytes(&msg_bytes) {
            Ok(_) => self.handshake_send_verack_message(stream),
            Err(_) => Err(NodeError::ErrorReceivingMessageInHandshake),
        }       
    }

    pub fn handle_verack_message(&self, hm: &HeaderMessage)-> Result<(), NodeError>{
        if hm.get_payload_size() != 0 && hm.get_command_name() != "verack\0\0\0\0\0\0" {
            return Err(NodeError::ErrorSendingMessageInHandshake);
        }
        
        Ok(())
    }

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