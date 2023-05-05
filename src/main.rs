

fn main() {
    use proyecto::config::Config;
    use std::env;

    let path: Vec<String> = env::args().collect();
    if !path.contains(&String::from("nodo.conf")) {
        return; //error
    }

    let config: Config = match(Config::from_path("nodo.conf")){
        Ok(config) => config,
        Err(_) => return, //error
    };
    
}





