use proyecto::node::*;
use proyecto::config::*;
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

    //log(config.log_file_path);

    Node::new(config);
}





