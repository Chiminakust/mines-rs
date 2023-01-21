extern crate argparse;

use argparse::{ArgumentParser, Store};

pub struct Config {
    pub rows: usize,
    pub cols: usize,
    pub mines_percent: f32,
    pub tile_width: usize,
    pub tile_height: usize,
    pub tile_gap: usize,
}

impl Config {
    pub fn build() -> Config {
        let mut rows: usize = 16;
        let mut cols: usize = 30;
        let mut mines_percent = 20.0;
        let mut tile_width = 30;
        let mut tile_height = 30;
        let mut tile_gap = 2;

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
            ap.refer(&mut tile_width)
                .add_option(&["-w", "--tile-width"], Store, "Width of a tile");
            ap.refer(&mut tile_height).add_option(
                &["-h", "--tile-height"],
                Store,
                "Height of a tile",
            );
            ap.refer(&mut tile_gap).add_option(
                &["-g", "--tile-gap"],
                Store,
                "Gap in pixels between tiles",
            );
            ap.parse_args_or_exit();
        }

        Config {
            rows,
            cols,
            mines_percent,
            tile_width,
            tile_height,
            tile_gap,
        }
    }
}
