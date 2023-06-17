use proyecto::node::*;
use proyecto::wallet::*;
use proyecto::wallet::*;
use proyecto::utils::config::*;
use std::env;
use std::thread;
use std::time::Duration;
use std::time::Instant;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return eprintln!("cantidad de argumentos inválida");
    }

    let config = match Config::from_path(args[1].as_str()) {
        Ok(config) => config,
        Err(error) => return eprintln!("{:?}", error),
    };

    let mut node = match Node::new(config) {
        Ok(node) => node,
        Err(error) => return eprintln!("{:?}", error),
    };
    node.initial_block_download().unwrap();

    node.logger.log("node, running".to_string());
    let message_receiver = match node.run() {
        Ok(message_receiver) => message_receiver,
        Err(error) => return eprintln!("{:?}", error),
    };

    //hace lo que quieras
    node.create_utxo_set()/* .map_err(|error| eprintln!("{:?}", error)) */;

    
    //node.get_utxo_balance([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]); // Llamamos a la función pasada como argumento
    
    //thread::sleep(Duration::from_secs(600));
    
    let a = [
        0x03, 0x57, 0xDD, 0x61, 0x2F, 0x9E, 0x51, 0xD0, 0x1C, 0x5C, 0xAC, 0x84, 0x35, 0xCB, 0x6C, 0x40, 0x15, 0x71, 0x50, 0x7C, 0xAF, 0xE3, 0x09, 0xE4, 0xA9, 0xBB, 0x48, 0xA4, 0x0B, 0x11, 0x8B, 0xF8, 0xFF
    ];
    
    let b = [
        0x8A, 0x39, 0x27, 0x84, 0x29, 0x20, 0x92, 0xB1, 0x94, 0x1F, 0x8A, 0x72, 0xB0, 0x94, 0x37, 0x16 , 0x04, 0x51, 0x8F, 0x55, 0x30, 0xA5, 0x8D, 0x66, 0xCA, 0x9D, 0xE3, 0x7E, 0x35, 0x6F, 0x8B, 0xBB
    ];

    let mut wallet = Wallet::new(a, b);
    
    node.set_wallet(wallet.get_pk_hash());
    let inicio = Instant::now();

    
    while(true){
        //wallet le habla al nodo
        node.update_utxo().unwrap();
        wallet.update(node.balance);
        println!("Balance: {}",node.balance);
        for (_,tx) in node.get_pending_tx().unwrap().iter(){
            for tx_out in &tx.tx_out{
                if tx_out.belongs_to(wallet.get_pk_hash()){
                    println!("hay en pending");
                }
            }
        }
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
    //vec<Wallets>


    //algo que recibe pedido del thread principal.
    //run_wallet();

    if let Err(error) = message_receiver.finish_receiving(){
        return eprintln!("{:?}", error)
        
    };
    node.logger.log(String::from("program finished gracefully"));
}
