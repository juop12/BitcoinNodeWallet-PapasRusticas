use crate::node::*;
use crate::utils::BtcError;
use crate::wallet::*;
use crate::utils::config::*;
use crate::utils::ui_communication_protocol::{UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse};
use std::time::Duration;
use std::time::Instant;
use glib::{Sender as GlibSender, Receiver as GlibReceiver};
use std::sync::mpsc;

pub fn initialize_node(args: Vec<String>)->Option<Node>{
    if args.len() != 2 {
        eprintln!("cantidad de argumentos inválida");
        return None;
    }

    let config = match Config::from_path(args[1].as_str()){
        Ok(config) => config,
        Err(error) => {
            eprintln!("ConfigError: {:?}", error);
            return None; 
        },
    };
    
    let mut node = match Node::new(config) {
        Ok(node) => node,
        Err(error) => {
            eprintln!("NodeError: {:?}", error);
            return None;
        },
    };
    
    if let Err(error) = node.initial_block_download(){
        eprintln!("Error IBD: {:?}", error);
        return None;
    };
    
    if let Err(error) = node.create_utxo_set(){
        eprintln!("Error creating UTXO set: {:?}", error);
        return None;
    };
    Some(node)
}


pub fn run(args: Vec<String>, sender_to_ui: GlibSender<UIResponse>, receiver: mpsc::Receiver<UIRequest>) {

    let mut node = match initialize_node(args){
        Some(node) => node,
        None => return sender_to_ui.send(UIResponse::ErrorInitializingNode).expect("Error sending message to UI"),
    };
    
    let message_receiver = match node.start_receiving_messages() {
        Ok(message_receiver) => message_receiver,
        Err(error) => return sender_to_ui.send(UIResponse::NodeRunningError(error)).expect("Error sending message to UI"),
    };

    node.logger.log(format!("node, running"));

    let mut wallet;
    
    loop{
        let ui_request = receiver.recv().expect("Error receiving message from UI");
        if let UIRequest::ChangeWallet(priv_key) = ui_request{
            //p asumo que las priv key son validas si tienen el tama;o valido
            wallet = match Wallet::from(priv_key){
                Ok(wallet) => wallet,
                Err(error) => return sender_to_ui.send(UIResponse::WalletError(error)).expect("Error sending message to UI"),
            };

            node.set_wallet(&mut wallet);
            wallet.send_wallet_info(&sender_to_ui);
            break;
        }
    }

    let mut last_update_time = Instant::now(); 

    let mut program_running = true;
    while program_running{
        if last_update_time.elapsed() < Duration::from_secs(5){
            if let Ok(request) = receiver.try_recv(){
                wallet = wallet.handle_ui_request(&mut node, request, &sender_to_ui, &mut program_running).expect("Error handeling ui request");
            }
        }else{
            node.update(&mut wallet).unwrap();
            wallet.send_wallet_info(&sender_to_ui);
            last_update_time = Instant::now();
            println!("Balance: {}",node.balance);
        }
    }

    if let Err(error) = message_receiver.finish_receiving(){
        return eprintln!("{:?}", error)
        
    };

    node.logger.log(String::from("program finished gracefully"));
    sender_to_ui.send(UIResponse::WalletFinished).expect("Error sending message to UI");
    
}