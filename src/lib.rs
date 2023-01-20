extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf;
use sdl2::video::{Window, WindowContext};

use rand::seq::IteratorRandom;
use std::error::Error;
use std::time::Duration;

mod config;

pub use crate::config::Config;

pub fn run(config: Config) -> Result<(), String> {
    println!(
        "Game with {} x {}, {}% mines",
        config.rows, config.cols, config.mines_percent
    );

    let minefield = Minefield::new(config.rows, config.cols, config.mines_percent);

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
                    x, y, mouse_btn, ..
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
                }
                _ => {}
            }
        }

        // draw on canvas
        minefield_renderer.clear_background(&mut canvas);
        minefield_renderer.draw_tiles(&mut canvas, &minefield);

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
    pub fn new(rows: usize, cols: usize, mines_percent: f32) -> Minefield {
        let mut tiles = vec![
            vec![
                Tile {
                    hidden: true,
                    content: TileContent::Danger(0),
                    flag: None
                };
                cols.try_into().unwrap()
            ];
            rows.try_into().unwrap()
        ];

        // place mines
        let n: usize = ((rows * cols) as f32 * (mines_percent / 100.0)) as usize;
        for i in (0..(rows * cols))
            .choose_multiple(&mut rand::thread_rng(), n)
            .iter()
        {
            let row = *i % rows;
            let col = *i / rows;
            tiles[row][col].set_as_mine();
        }

        // compute danger indicators of tiles
        for i in 0..(rows * cols) {
            let row = i % rows;
            let col = i / rows;

            // skip if mine
            if tiles[row][col].content == TileContent::Mine {
                continue;
            }

            // TODO: there has to be a better way, but this works for now
            // check the 8 neighbours
            let mut danger_level = 0;
            for j in -1..=1 {
                for k in -1..=1 {
                    // do not include current tile
                    if (j, k) == (0, 0) {
                        continue;
                    }

                    // check boundaries
                    let x = row as i32 + j;
                    let y = col as i32 + k;
                    if 0 > x || x >= rows as i32 {
                        continue;
                    }
                    if 0 > y || y >= cols as i32 {
                        continue
                    }

                    if tiles[x as usize][y as usize].content == TileContent::Mine {
                        danger_level += 1;
                    }
                }
            }

            tiles[row][col].set_danger_level(danger_level);
        }

        Minefield { tiles, rows, cols }
    }

    fn tile_to_indices(&self, tile_number: usize) -> (usize, usize) {
        let row = tile_number % self.rows;
        let col = tile_number / self.rows;
        (row, col)
    }

    pub fn uncover_tile(&self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].uncover();
    }

    pub fn flag_tile(&self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].flag();
    }

    pub fn get_tile_content(&self, tile_number: usize) -> TileContent {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].content.clone()
    }

    pub fn place_mine_on_tile(&mut self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].set_as_mine();
    }
}

#[derive(Clone)]
struct Tile {
    hidden: bool,
    content: TileContent,
    flag: Option<Flag>,
}

#[derive(Clone, Debug, PartialEq)]
enum TileContent {
    Mine,
    Danger(i32),
}

#[derive(Clone, Debug)]
enum Flag {
    Mine,
    Question,
}

impl Tile {
    pub fn uncover(&self) {
        println!("i am uncovered, {:?}", self.content);
    }

    pub fn flag(&self) {
        println!("i am flagged, {:?}", self.content);
    }

    pub fn set_as_mine(&mut self) {
        self.content = TileContent::Mine;
    }

    pub fn set_danger_level(&mut self, danger_level: i32) {
        self.content = TileContent::Danger(danger_level);
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

    pub fn draw_tiles(
        &self,
        canvas: &mut Canvas<Window>,
        minefield: &Minefield,
    ) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(Color::RGB(230, 230, 230));

        for (i, draw_zone) in self.tiles_coords.iter().enumerate() {
            canvas.fill_rect(*draw_zone).unwrap();


            match minefield.get_tile_content(i) {
                TileContent::Danger(i) => match i {
                    0 => canvas.copy(&self.textures.tile_danger_0, None, Some(*draw_zone))?,
                    1 => canvas.copy(&self.textures.tile_danger_1, None, Some(*draw_zone))?,
                    2 => canvas.copy(&self.textures.tile_danger_2, None, Some(*draw_zone))?,
                    3 => canvas.copy(&self.textures.tile_danger_3, None, Some(*draw_zone))?,
                    4 => canvas.copy(&self.textures.tile_danger_4, None, Some(*draw_zone))?,
                    5 => canvas.copy(&self.textures.tile_danger_5, None, Some(*draw_zone))?,
                    6 => canvas.copy(&self.textures.tile_danger_6, None, Some(*draw_zone))?,
                    7 => canvas.copy(&self.textures.tile_danger_7, None, Some(*draw_zone))?,
                    8 => canvas.copy(&self.textures.tile_danger_8, None, Some(*draw_zone))?,
                    _ => (),
                },
                TileContent::Mine => {
                    canvas.copy(&self.textures.tile_mine, None, Some(*draw_zone))?;
                }
            }
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
    tile_danger_0: Texture,
    tile_danger_1: Texture,
    tile_danger_2: Texture,
    tile_danger_3: Texture,
    tile_danger_4: Texture,
    tile_danger_5: Texture,
    tile_danger_6: Texture,
    tile_danger_7: Texture,
    tile_danger_8: Texture,
    tile_flag_mine: Texture,
    tile_flag_question: Texture,
    tile_mine: Texture,
    tile_blank: Texture,
}

impl MinefieldRendererTextures {
    pub fn new(
        font: ttf::Font,
        texture_creator: &TextureCreator<WindowContext>,
    ) -> Result<MinefieldRendererTextures, Box<dyn Error>> {
        let tile_danger_0 = texture_creator
            .create_texture_from_surface(
                font.render("0")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_1 = texture_creator
            .create_texture_from_surface(
                font.render("1")
                    .blended(Color::RGB(0, 200, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_2 = texture_creator
            .create_texture_from_surface(
                font.render("2")
                    .blended(Color::RGB(0, 200, 200))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_3 = texture_creator
            .create_texture_from_surface(
                font.render("3")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_4 = texture_creator
            .create_texture_from_surface(
                font.render("4")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_5 = texture_creator
            .create_texture_from_surface(
                font.render("5")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_6 = texture_creator
            .create_texture_from_surface(
                font.render("6")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_7 = texture_creator
            .create_texture_from_surface(
                font.render("7")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_danger_8 = texture_creator
            .create_texture_from_surface(
                font.render("8")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_flag_mine = texture_creator
            .create_texture_from_surface(
                font.render("F")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_flag_question = texture_creator
            .create_texture_from_surface(
                font.render("?")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_mine = texture_creator
            .create_texture_from_surface(
                font.render("*")
                    .blended(Color::RGB(255, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let tile_blank = texture_creator
            .create_texture_from_surface(
                font.render(" ")
                    .blended(Color::RGB(0, 0, 0))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        Ok(MinefieldRendererTextures {
            tile_danger_0,
            tile_danger_1,
            tile_danger_2,
            tile_danger_3,
            tile_danger_4,
            tile_danger_5,
            tile_danger_6,
            tile_danger_7,
            tile_danger_8,
            tile_flag_mine,
            tile_flag_question,
            tile_mine,
            tile_blank,
        })
    }
}
