use proyecto::start_app;
use node::run::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    run(args);
    start_app();
}
