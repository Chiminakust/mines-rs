extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf;
use sdl2::video::{Window, WindowContext};

use std::error::Error;
use std::time::Duration;

mod config;

pub use crate::config::Config;

pub fn run(config: Config) -> Result<(), String> {
    println!("Game with {} x {}", config.rows, config.cols);

    let minefield = Minefield::new(config.rows, config.cols);

    let win_width: u32 = config.cols as u32 * 27; // arbitrary
    let win_height: u32 = (win_width / config.cols as u32) * config.rows as u32;

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
    let minefield_renderer = MinefieldRenderer::new(&canvas, &ttf_context, &minefield).unwrap();
    // need to create a texture for the font

    minefield_renderer.clear_background(&mut canvas);

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
                } => {
                    let point = Point::new(x, y);
                    if let Some(clicked_tile) = minefield_renderer.get_tile_index(point) {
                        match mouse_btn {
                            MouseButton::Left => {
                                minefield.uncover_tile(clicked_tile);
                            }
                            MouseButton::Right => {
                                minefield.flag_tile(clicked_tile);
                            }
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }

        // draw on canvas
        minefield_renderer.clear_background(&mut canvas);
        minefield_renderer.draw_tiles(&mut canvas);

        // refresh displayed canvas
        canvas.present();

        // frame rate limit
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        // The rest of the game loop goes here...
    }

    Ok(())
}

struct Minefield {
    tiles: Vec<Vec<Tile>>,
    rows: usize,
    cols: usize,
}

impl Minefield {
    pub fn new(rows: usize, cols: usize) -> Minefield {
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

        Minefield { tiles, rows, cols }
    }

    fn tile_to_indices(&self, tile_number: usize) -> (usize, usize) {
        let row = tile_number % self.rows;
        let col = tile_number / self.rows;
        (row, col)
    }

    pub fn uncover_tile(&self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        println!("uncovering tile {},{}", row, col);
        self.tiles[row][col].uncover();
    }

    pub fn flag_tile(&self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        println!("flagging tile {},{}", row, col);
        self.tiles[row as usize][col as usize].flag();
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

impl Tile {
    pub fn uncover(&self) {
        println!("i am uncovered");
    }

    pub fn flag(&self) {
        println!("i am flagged");
    }
}

struct MinefieldRenderer {
    tiles_coords: Vec<Rect>,
    textures: MinefieldRendererTextures,
}

impl MinefieldRenderer {
    pub fn new(
        canvas: &Canvas<Window>,
        ttf_context: &ttf::Sdl2TtfContext,
        minefield: &Minefield,
    ) -> Result<MinefieldRenderer, Box<dyn Error>> {
        // compute where the tiles will be on the screen
        let rows = minefield.rows;
        let cols = minefield.cols;
        let origin = (10, 10);
        let tile_size = (20, 20);
        let tiles_coords = (0..(rows * cols))
            .map(|x: usize| {
                Rect::new(
                    (origin.0 + ((x / rows) * (tile_size.0 + 2)))
                        .try_into()
                        .unwrap(),
                    (origin.1 + ((x % rows) * (tile_size.1 + 2)))
                        .try_into()
                        .unwrap(),
                    tile_size.0.try_into().unwrap(),
                    tile_size.1.try_into().unwrap(),
                )
            })
            .collect();

        let mut font =
            ttf_context.load_font("assets/fonts/lm-mono-font/Lmmono12Regular-K7qoZ.otf", 128)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        // texture creator for later
        let texture_creator = canvas.texture_creator();

        let textures = MinefieldRendererTextures::new(font, &texture_creator).unwrap();

        Ok(MinefieldRenderer {
            tiles_coords,
            textures,
        })
    }

    pub fn draw_tiles(&self, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(Color::RGB(230, 230, 230));

        for draw_zone in self.tiles_coords.iter() {
            canvas.fill_rect(*draw_zone).unwrap();
            canvas.copy(&self.textures.tile_danger_1, None, Some(*draw_zone))?;
        }

        Ok(())
    }

    pub fn clear_background(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.clear();
    }

    pub fn get_tile_index(&self, point: Point) -> Option<usize> {
        for (i, draw_zone) in self.tiles_coords.iter().enumerate() {
            if draw_zone.contains_point(point) {
                return Some(i);
            }
        }
        None
    }
}

struct MinefieldRendererTextures {
    tile_danger_1: Texture,
    tile_danger_2: Texture,
}

impl MinefieldRendererTextures {
    pub fn new(
        font: ttf::Font,
        texture_creator: &TextureCreator<WindowContext>,
    ) -> Result<MinefieldRendererTextures, Box<dyn Error>> {

        let tile_surface_1 = font
            .render("1")
            .blended(Color::RGB(0, 0, 0))
            .map_err(|e| e.to_string())?;
        let tile_danger_1 = texture_creator
            .create_texture_from_surface(&tile_surface_1)
            .map_err(|e| e.to_string())?;

        let tile_surface_2 = font
            .render("2")
            .blended(Color::RGB(0, 0, 0))
            .map_err(|e| e.to_string())?;
        let tile_danger_2 = texture_creator
            .create_texture_from_surface(&tile_surface_2)
            .map_err(|e| e.to_string())?;

        Ok(MinefieldRendererTextures {
            tile_danger_1,
            tile_danger_2,
        })
    }
}
