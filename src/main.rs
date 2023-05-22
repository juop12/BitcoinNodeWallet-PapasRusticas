use proyecto::utils::config::*;
// use proyecto::utils::log::*;
use proyecto::node::*;
use std::env;



fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return eprintln!("cantidad de argumentos inválida");
    }

    let config = match Config::from_path(args[1].as_str()){
        Ok(config) => config,
        Err(error) => return eprintln!("{:?}", error),
    };

    let mut node = match Node::new(config){
        Ok(node) => node,
        Err(error) => return eprintln!("{:?}", error),
    };

    match node.initial_block_download(){
        Ok(_) => {
            println!("IBD completed");
        },
        Err(err) => {
            println!("IBD failed: {:?}", err);
        },
    };

    match node.create_utxo_set(){
        Some(utxo_set) => {
            println!("utxo_set Len:: {}\n\n", utxo_set.len());
        },
        None => {
            println!("utxo_set is None");
        },
    };

    node.run();
}
