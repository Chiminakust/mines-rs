extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

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
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", win_width, win_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    // need to create a texture for the font (i think?)
    let texture_creator = canvas.texture_creator();

    // load a font
    let mut font = ttf_context.load_font("assets/fonts/lm-mono-font/Lmmono12Regular-K7qoZ.otf", 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);
    let surface = font
        .render("hello ttf!1234567890")
        .blended(Color::RGB(0, 0, 0))
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();

    let target = Rect::new(40, 40, 300, 100);

    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        // event loop
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

        // clear screen
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.clear();

        // draw rectangle
        canvas.set_draw_color(Color::RGB(0, 255, 255));

        let tile_w = 20;

        let tile_h = tile_w;
        let tile_surface_1 = font
            .render("1")
            .blended(Color::RGB(0, 0, 0))
            .map_err(|e| e.to_string())?;
        let tile_texture = texture_creator
            .create_texture_from_surface(&tile_surface_1)
            .map_err(|e| e.to_string())?;

        for (col, line) in minefield.tiles.iter().enumerate() {
            for (row, tile) in line.iter().enumerate() {
                let draw_zone = Rect::new(
                    (10 + (row * tile_w)).try_into().unwrap(),
                    (10 + (col * tile_h)).try_into().unwrap(),
                    tile_w.try_into().unwrap(),
                    tile_h.try_into().unwrap()
                );
                canvas.draw_rect(draw_zone).unwrap();
                canvas.copy(&tile_texture, None, Some(draw_zone))?;
            }
        }

        // canvas.draw_rect(Rect::new(10, 10, 20, 20)).unwrap();

        // // draw text
        // canvas.copy(&texture, None, Some(target))?;

        // display canvas
        canvas.present();


        // frame rate limit
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
