use crate::node::*;
use crate::utils::{ui_communication, BtcError};
use crate::wallet::*;
use crate::utils::config::*;
use crate::utils::ui_communication::{UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse};
//use user_interface::start_app;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use bs58::*;
use glib::{Sender as GlibSender, Receiver as GlibReceiver};
use std::sync::mpsc;

fn initialize_node(args: Vec<String>)->Option<Node>{
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

    let node = initialize_node(args);
    let mut node = match node{
        Some(node) => node,
        None => {
            sender_to_ui.send(UIResponse::ErrorInitializingNode).unwrap_or_else(|e| eprintln!("Error saending message to UI: {:?}", e));
            return;
        }
    };
    
    let message_receiver = match node.run() {
        Ok(message_receiver) => message_receiver,
        Err(error) => {
            sender_to_ui.send(UIResponse::NodeRunningError(error)).unwrap_or_else(|e| eprintln!("Error saending message to UI: {:?}", e));
            return;
        },
    };

    node.logger.log(format!("node, running"));

    
    while true{
        let ui_request = receiver.recv().expect("Error receiving message from UI");
        if let UIRequest::ChangeWallet(pub_key, priv_key) = ui_request{
            Wallet::create(pub_key, priv_key);
        }
        
    }
    
    // Inicio de wallets.
    //=========================================================================

    let pub_key = [
        0x03, 0x57, 0xDD, 0x61, 0x2F, 0x9E, 0x51, 0xD0, 0x1C, 0x5C, 0xAC, 0x84, 0x35, 0xCB, 0x6C, 0x40, 0x15, 0x71, 0x50, 0x7C, 0xAF, 0xE3, 0x09, 0xE4, 0xA9, 0xBB, 0x48, 0xA4, 0x0B, 0x11, 0x8B, 0xF8, 0xFF
    ];
    
    let priv_key = [
        0x8A, 0x39, 0x27, 0x84, 0x29, 0x20, 0x92, 0xB1, 0x94, 0x1F, 0x8A, 0x72, 0xB0, 0x94, 0x37, 0x16 , 0x04, 0x51, 0x8F, 0x55, 0x30, 0xA5, 0x8D, 0x66, 0xCA, 0x9D, 0xE3, 0x7E, 0x35, 0x6F, 0x8B, 0xBB
    ];
        
    
    //let pub_key = get_bytes_from_hex("0357DD612F9E51D01C5CAC8435CB6C401571507CAFE309E4A9BB48A40B118BF8FF");
    //let priv_key = get_bytes_from_hex("8A392784292092B1941F8A72B094371604518F5530A58D66CA9DE37E356F8BBB");
    
    let receiver_address = "miHXVsyAd3dG78Ri78NUmAfCyoHXaYkibp";
        
    // Address de prueba para enviar.
    println!("por crear la walle");
    let mut wallet = Wallet::new(pub_key, priv_key);
    
    println!("por setee la wallet");
    node.set_wallet(wallet.get_pk_hash());
    
    let address_bytes = bs58::decode(receiver_address).into_vec().unwrap();
    let mut address: [u8;25] = [0;25];
    address.copy_from_slice(&address_bytes);
    
    println!("por mandar la transaccion");
    
    //thread::sleep(Duration::from_secs(60));
    wallet.create_transaction(&mut node, 70000, 30000, address).expect("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    
    let mut inicio = Instant::now(); 
    
    while(true){
        //wallet le habla al nodo
        node.update_utxo().unwrap();
        wallet.update(node.balance);
        
        thread::sleep(Duration::from_secs(5));
        //p esta version esta mejor porque despues podemos poner la respuesta a la ui ahi, para que sea en tiempo real con un time receive, pero que la wallet se ac
        //if inicio.elapsed() > Duration::from_secs(5){
        //    inicio = Instant::now();
        //}
        println!("Balance: {}",node.balance);
        
        //for (_,tx) in node.get_pending_tx().unwrap().iter(){
        //    for tx_out in &tx.tx_out{
        //        if tx_out.belongs_to(wallet.get_pk_hash()){
        //            println!("hay en pending");
        //        }
        //    }
        //}
        
    }

    /*
    while(){
        wallet.ciclo_wallet(&node) => {
            //recibis de la ui
            //if ui quiere cambiar wallet
            //      wallet llama a nodo.change_wallet(), se dropea y se instancia una nueva

            //le pedis cosas al nodo
        }
        node.actualizar_wallet(&wallet) => {
            //se actualiza el nodo a si mismo, tiene guardado cual es el ultimo bloque "procesado", y procesa los que le faltan
            //wallet.actualizar(transaction);
            
            //tengo un nuevo pending
            //me llego una transaccion a actualizar
        }
    }*/

    if let Err(error) = message_receiver.finish_receiving(){
        return eprintln!("{:?}", error)
        
    };
  
    node.logger.log(String::from("program finished gracefully"));
}
