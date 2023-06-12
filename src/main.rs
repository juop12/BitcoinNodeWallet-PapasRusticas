use proyecto::node::*;
use proyecto::utils::config::*;
use std::env;
use std::thread;
use std::time::Duration;

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
    thread::sleep(Duration::from_secs(600));

    if let Err(error) = message_receiver.finish_receiving(){
        return eprintln!("{:?}", error)
        
    };
    node.logger.log(String::from("program finished gracefully"));
}
