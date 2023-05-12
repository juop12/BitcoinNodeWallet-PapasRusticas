use proyecto::config::*;
use proyecto::node::*;
use proyecto::log::*;
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

    let log = match Logger::from_path(&config.log_path){
        Ok(log) => log,
        Err(error) => return eprintln!("{:?}", error),
    };

    Node::new(log, config);
}
