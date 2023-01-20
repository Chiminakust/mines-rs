extern crate argparse;

use argparse::{ArgumentParser, Store};

pub struct Config {
    pub rows: usize,
    pub cols: usize,
    pub mines_percent: f32,
}

impl Config {
    pub fn build() -> Config {
        let mut rows: usize = 16;
        let mut cols: usize = 30;
        let mut mines_percent = 20.0;

        {
            let mut ap = ArgumentParser::new();
            ap.set_description("A mines clone written in Rust.");
            ap.refer(&mut rows)
                .add_option(&["-r", "--rows"], Store, "Number of rows");
            ap.refer(&mut cols)
                .add_option(&["-c", "--cols"], Store, "Number of columns");
            ap.refer(&mut mines_percent).add_option(
                &["-p", "--percent"],
                Store,
                "Percentage of mines",
            );
            ap.parse_args_or_exit();
        }

        Config {
            rows,
            cols,
            mines_percent,
        }
    }
}
