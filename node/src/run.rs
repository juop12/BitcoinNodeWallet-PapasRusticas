use crate::node::*;
use crate::utils::BtcError;
use crate::wallet::*;
use crate::utils::config::*;
use crate::utils::ui_communication_protocol::{UIToWalletCommunication as UIRequest, WalletToUICommunication as UIResponse};
use std::time::Duration;
use std::time::Instant;
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
            break;
        }
    }

    let mut last_update_time = Instant::now(); 

    let mut program_running = true;
    while program_running{
        if last_update_time.elapsed() > Duration::from_secs(5){
            if let Ok(request) = receiver.try_recv(){
                wallet = wallet.handle_ui_request(&mut node, request, &sender_to_ui, &mut program_running).expect("Error handeling ui request");
            }
        }else{
            node.update(&mut wallet).unwrap();
            last_update_time = Instant::now();
            println!("Balance: {}",node.balance);
        }
    }

    // Inicio de wallets.
    //=========================================================================
    
    //let pub_key = get_bytes_from_hex("0357DD612F9E51D01C5CAC8435CB6C401571507CAFE309E4A9BB48A40B118BF8FF");
    //let priv_key = get_bytes_from_hex("8A392784292092B1941F8A72B094371604518F5530A58D66CA9DE37E356F8BBB");
    /* 
    let receiver_address = "miHXVsyAd3dG78Ri78NUmAfCyoHXaYkibp";
    let priv_key = "cSDPYr9FfseHx8jbjrnz9ryERswMkv6vKSccomu1ShfrJXj2d65Z";
    
    // Address de prueba para enviar.
    println!("por crear la walle");
    let mut wallet = Wallet::from(priv_key.to_string()).unwrap();
    
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
    }
    */

    if let Err(error) = message_receiver.finish_receiving(){
        return eprintln!("{:?}", error)
        
    };
    
    node.logger.log(String::from("program finished gracefully"));
}
