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

    let mut minefield = Minefield::new(config.rows, config.cols, config.mines_percent);

    let win_width = ((config.tile_width + config.tile_gap) * config.cols) + 2 * config.origin.0;
    let win_height = ((config.tile_height + config.tile_gap) * config.rows) + 2 * config.origin.1;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", win_width as u32, win_height as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let minefield_renderer = MinefieldRenderer::new(
        &canvas,
        &ttf_context,
        &minefield,
        (config.tile_width, config.tile_height),
        config.tile_gap,
        config.origin,
    )
    .unwrap();

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
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    minefield.reset();
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
    mines_percent: f32,
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

        let mut minefield = Minefield {
            tiles,
            rows,
            cols,
            mines_percent,
        };

        minefield.reset();

        minefield
    }

    pub fn reveal(&mut self) {
        for col in self.tiles.iter_mut() {
            for tile in col.iter_mut() {
                (*tile).uncover();
            }
        }
    }

    pub fn reset(&mut self) {
        let total_tiles = self.rows * self.cols;

        // reset all tiles
        for i in 0..total_tiles {
            self.reset_tile(i);
        }

        // place mines
        let n: usize = (total_tiles as f32 * (self.mines_percent / 100.0)) as usize;
        for i in (0..total_tiles)
            .choose_multiple(&mut rand::thread_rng(), n)
            .iter()
        {
            let (row, col) = self.tile_to_indices(*i);
            self.tiles[row][col].set_as_mine();
        }

        // compute danger indicators of tiles
        for i in 0..total_tiles {
            let (row, col) = self.tile_to_indices(i);

            // skip if tile is a mine
            if self.tiles[row][col].content == TileContent::Mine {
                continue;
            }

            let mut danger_level = 0;

            for (x, y) in self.get_neighbours(i).iter() {
                if self.tiles[*x][*y].content == TileContent::Mine {
                    danger_level += 1;
                }
            }

            self.tiles[row][col].set_danger_level(danger_level);
        }
    }

    fn get_neighbours(&self, tile_number: usize) -> Vec<(usize, usize)> {
        let (row, col) = self.tile_to_indices(tile_number);
        let mut neighbours = vec![];

        // TODO: there has to be a better way, but this works for now
        for j in -1..=1 {
            for k in -1..=1 {
                // do not include current tile
                if (j, k) == (0, 0) {
                    continue;
                }

                // check boundaries
                let x = row as i32 + j;
                let y = col as i32 + k;
                if 0 > x || x >= self.rows as i32 {
                    continue;
                }
                if 0 > y || y >= self.cols as i32 {
                    continue;
                }

                neighbours.push((x as usize, y as usize));
            }
        }

        neighbours
    }

    fn tile_to_indices(&self, tile_number: usize) -> (usize, usize) {
        let row = tile_number % self.rows;
        let col = tile_number / self.rows;
        (row, col)
    }

    fn indices_to_tile(&self, row: usize, col: usize) -> usize {
        col * self.rows + row
    }

    pub fn uncover_tile(&mut self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].uncover();
        match self.get_tile_content(tile_number) {
            TileContent::Mine => {
                println!("BOOM from mine {},{}", row, col);
                self.reveal();
            }
            TileContent::Danger(0) => {
                self.discover(tile_number);
            }
            _ => {}
        }
    }

    pub fn hide_tile(&mut self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].hide();
    }

    pub fn reset_tile(&mut self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].reset();
    }

    pub fn flag_tile(&mut self, tile_number: usize) {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].flag();
    }

    pub fn get_tile_content(&self, tile_number: usize) -> TileContent {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].content.clone()
    }

    pub fn get_tile_flag(&self, tile_number: usize) -> Option<Flag> {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].flag.clone()
    }

    pub fn tile_is_hidden(&self, tile_number: usize) -> bool {
        let (row, col) = self.tile_to_indices(tile_number);
        self.tiles[row][col].hidden
    }

    fn discover(&mut self, tile_number: usize) {
        for (x, y) in self.get_neighbours(tile_number).iter() {
            let neighbour_index = self.indices_to_tile(*x, *y);

            // skip if already revealed
            if !self.tile_is_hidden(neighbour_index) {
                continue;
            }

            self.uncover_tile(neighbour_index);
        }
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
    pub fn uncover(&mut self) {
        // println!("i am uncovered, {:?} {:?}", self.content, self.flag);
        self.hidden = false;
    }

    pub fn hide(&mut self) {
        self.hidden = true;
    }

    pub fn flag(&mut self) {
        // println!("i am flagged, {:?}, {:?}", self.content, self.flag);
        if let Some(flag) = self.flag.clone() {
            match flag {
                Flag::Mine => self.flag = Some(Flag::Question),
                Flag::Question => self.flag = None,
            }
        } else {
            self.flag = Some(Flag::Mine);
        }
    }

    fn reset_flag(&mut self) {
        self.flag = None;
    }

    pub fn set_as_mine(&mut self) {
        self.content = TileContent::Mine;
    }

    pub fn set_danger_level(&mut self, danger_level: i32) {
        self.content = TileContent::Danger(danger_level);
    }

    pub fn reset(&mut self) {
        self.hide();
        self.set_danger_level(0);
        self.reset_flag();
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
        tile_size: (usize, usize),
        tile_gap: usize,
        origin: (usize, usize),
    ) -> Result<MinefieldRenderer, Box<dyn Error>> {
        // compute where the tiles will be on the screen
        let rows = minefield.rows;
        let cols = minefield.cols;
        let tiles_coords = (0..(rows * cols))
            .map(|x: usize| {
                Rect::new(
                    (origin.0 + ((x / rows) * (tile_size.0 + tile_gap)))
                        .try_into()
                        .unwrap(),
                    (origin.1 + ((x % rows) * (tile_size.1 + tile_gap)))
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

            if minefield.tile_is_hidden(i) {
                if let Some(flag) = minefield.get_tile_flag(i) {
                    match flag {
                        Flag::Mine => {
                            canvas.copy(&self.textures.tile_flag_mine, None, Some(*draw_zone))?
                        }
                        Flag::Question => canvas.copy(
                            &self.textures.tile_flag_question,
                            None,
                            Some(*draw_zone),
                        )?,
                    }
                } else {
                    canvas.copy(&self.textures.tile_blank, None, Some(*draw_zone))?;
                }
            } else {
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
