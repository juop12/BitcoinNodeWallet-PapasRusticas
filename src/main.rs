use proyecto::start_app;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    start_app(args);
}
