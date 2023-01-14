use std::process;

use mines::Config;

pub fn main() {

    let config = Config::build();

    if let Err(e) = mines::run(config) {
        eprintln!("Game ran into an error: {e}");
        process::exit(1);
    }
}
