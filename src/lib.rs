extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use std::time::Duration;

mod config;

pub use crate::config::Config;

pub fn run(config: Config) -> Result<(), String> {
    println!("Game with {} x {}", config.rows, config.cols);

    let minefield = Minefield::new(config.rows, config.cols);

    let win_width: u32 = 800;
    let win_height: u32 = (win_width / config.cols) * config.rows;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", win_width, win_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn,
                    clicks,
                    ..
                } => match mouse_btn {
                    MouseButton::Left => {
                        println!("x,y = {},{}, button = left, clicks = {}", x, y, clicks)
                    }
                    MouseButton::Right => {
                        println!("x,y = {},{}, button = right, clicks = {}", x, y, clicks)
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        canvas.clear();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}

struct Minefield {
    tiles: Vec<Vec<Tile>>,
}

impl Minefield {
    pub fn new(rows: u32, cols: u32) -> Minefield {
        let tiles = vec![
            vec![
                Tile {
                    hidden: true,
                    content: TileContent::Blank,
                    flag: None
                };
                cols.try_into().unwrap()
            ];
            rows.try_into().unwrap()
        ];

        Minefield { tiles }
    }
}

#[derive(Clone)]
struct Tile {
    hidden: bool,
    content: TileContent,
    flag: Option<Flag>,
}

#[derive(Clone)]
enum TileContent {
    Blank,
    Mine,
    Danger(i32),
}

#[derive(Clone)]
enum Flag {
    Mine,
    Question,
}
