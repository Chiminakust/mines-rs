use std::process;

use mines::Config;

pub fn main() {

    let config = Config::build().unwrap_or_else(|err| {
        eprintln!("Could not parse arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = mines::run(config) {
        eprintln!("Game ran into an error: {e}");
        process::exit(1);
    }
}
