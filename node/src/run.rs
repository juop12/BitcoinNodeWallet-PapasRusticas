use crate::utils::ui_communication_protocol::{
    UIToWalletCommunication as UIRequest, 
    WalletToUICommunication as UIResponse
};
use crate::{
    node::*,
    wallet::*,
    utils::config::*,
    utils::WalletError,
};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use glib::Sender as GlibSender;

const REFRESH_RATE: Duration = Duration::from_secs(5);


///-
pub fn initialize_node(args: Vec<String>) -> Option<Node>{
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

    node.start_receiving_messages();

    Some(node)
}

///-
fn exit_program(sender_to_ui: GlibSender<UIResponse>, message: UIResponse){
    match message {
        UIResponse::WalletError(error) => {
            if let WalletError::ErrorSendingToUI = error{
                return eprintln!("Error sending_to_ui: {:?}", error);
            } 
        },
        _ => {
            if let Err(error) = sender_to_ui.send(message){
                eprintln!("Error sending_to_ui: {:?}", error);
            }
        }
    }    
}

///-
fn get_first_wallet(node: &mut Node, receiver: &mpsc::Receiver<UIRequest>, sender_to_ui: &GlibSender<UIResponse>) -> Result<Wallet, WalletError>{
    loop{
        let ui_request = receiver.recv().map_err(|_| WalletError::ErrorReceivingFromUI)?;
        
        if let UIRequest::ChangeWallet(priv_key) = ui_request{

            let mut wallet = Wallet::from(priv_key)?;

            node.set_wallet(&mut wallet).map_err(|_| WalletError::ErrorSettingWallet)?;

            wallet.send_wallet_info(&sender_to_ui)?;
            
            return Ok(wallet);
        }
    }
}

///-
fn main_loop(node: &mut Node, mut wallet: Wallet, receiver: &mpsc::Receiver<UIRequest>, sender_to_ui: &GlibSender<UIResponse>) -> Result<(), WalletError>{
    let mut last_update_time = Instant::now(); 
    let mut program_running = true;
    
    while program_running {
        
        if last_update_time.elapsed() < REFRESH_RATE {
            if let Ok(request) = receiver.try_recv(){
                wallet = wallet.handle_ui_request(node, request, &sender_to_ui, &mut program_running)?;
            }
        } else {
            
            node.update(&mut wallet).map_err(|_| WalletError::ErrorUpdatingWallet)?;
            wallet.send_wallet_info(&sender_to_ui)?;
            
            last_update_time = Instant::now();
            println!("Balance: {}",node.balance);
        }
    }
    
    Ok(())
}

///-
pub fn run(args: Vec<String>, sender_to_ui: GlibSender<UIResponse>, receiver: mpsc::Receiver<UIRequest>) {

    let mut node = match initialize_node(args){
        Some(node) => node,
        None => return exit_program(sender_to_ui, UIResponse::ErrorInitializingNode),
    };
    
    if let Err(error) = sender_to_ui.send(UIResponse::FinishedInitializingNode){
        return eprintln!("Error sending_to_ui: {:?}", error);
    }

    node.logger.log(format!("node, running"));
    
    let wallet = match get_first_wallet(&mut node, &receiver, &sender_to_ui){
        Ok(wallet) => wallet,
        Err(error) => return exit_program(sender_to_ui, UIResponse::WalletError(error)),
    };

    if let Err(error) = main_loop(&mut node, wallet, &receiver, &sender_to_ui){
        return exit_program(sender_to_ui, UIResponse::WalletError(error));
    };

    node.logger.log(format!("program finished gracefully"));
    exit_program(sender_to_ui, UIResponse::WalletFinished);
}