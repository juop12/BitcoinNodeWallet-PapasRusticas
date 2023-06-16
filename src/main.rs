use proyecto::node::*;
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
    node.create_utxo_set();
    let inicio = Instant::now();
    node.get_utxo_balance([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]); // Llamamos a la función pasada como argumento
    let duracion = inicio.elapsed();

    println!("La función tardó: {:?}", duracion);
    thread::sleep(Duration::from_secs(600));

    

    /*
    while(){
        wallet.ciclo_wallet(&node) => {
            node.haceralgo()
            //recibis de la ui
            //haces ese algo
            //le pedis cosas al nodo
        }
        node.actualizar_wallet(&wallet) => {
            wallet.actualizar(transaction);
            
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
