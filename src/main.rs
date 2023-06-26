use proyecto::start_app;
use std::env;

/// Starts the application
fn main() {
    let args: Vec<String> = env::args().collect();
    start_app(args);
}
