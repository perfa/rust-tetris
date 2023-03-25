extern crate sdl2;

use crate::engine::piece::{Direction, Piece, Rotation};
use crate::engine::{Board, Coordinate, Engine};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::ttf::Font;
use sdl2::{event::Event, render::WindowCanvas};
use std::time::Duration;

enum GameState {
    TitleScreen,
    Playing,
    Paused,
    GameOver,
}

struct Matrix {
    x: i32,
    y: i32,
}

impl Matrix {
    const SQUARE_SIZE: i32 = 28;
    pub fn new(width: i32, top_offset: i32) -> Self {
        let x = (width - (10 * Matrix::SQUARE_SIZE)) / 2;
        Matrix {
            x: x,
            y: top_offset,
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, engine: &Engine) {
        if let Some(cursor) = engine.cursor.as_ref() {
            let minos = cursor.get_cells();
            for mino in &minos {
                if mino.y < 0 {
                    continue;
                }
                canvas.set_draw_color(Color::RGB(150, 200, 150));
                canvas
                    .fill_rect(Rect::new(
                        2 + self.x + (mino.x as i32) * Matrix::SQUARE_SIZE,
                        2 + self.y + (mino.y as i32) * Matrix::SQUARE_SIZE,
                        Matrix::SQUARE_SIZE as u32 - 4,
                        Matrix::SQUARE_SIZE as u32 - 4,
                    ))
                    .unwrap();
                canvas.set_draw_color(Color::RGB(200, 200, 200));
                canvas
                    .draw_rect(Rect::new(
                        self.x + (mino.x as i32) * Matrix::SQUARE_SIZE,
                        self.y + (mino.y as i32) * Matrix::SQUARE_SIZE,
                        Matrix::SQUARE_SIZE as u32,
                        Matrix::SQUARE_SIZE as u32,
                    ))
                    .unwrap();
            }
        }

        for cell in engine.get_pile() {
            canvas.set_draw_color(Color::RGB(100, 150, 100));
            canvas
                .fill_rect(Rect::new(
                    2 + self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    2 + self.y + (cell.y as i32) * Matrix::SQUARE_SIZE,
                    Matrix::SQUARE_SIZE as u32 - 4,
                    Matrix::SQUARE_SIZE as u32 - 4,
                ))
                .unwrap();
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas
                .draw_rect(Rect::new(
                    self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    self.y + (cell.y as i32) * Matrix::SQUARE_SIZE,
                    Matrix::SQUARE_SIZE as u32,
                    Matrix::SQUARE_SIZE as u32,
                ))
                .unwrap();
        }

        for cell in engine.get_marked() {
            canvas.set_draw_color(Color::RGB(10, 15, 10));
            canvas
                .fill_rect(Rect::new(
                    2 + self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    2 + self.y + (cell.y as i32) * Matrix::SQUARE_SIZE,
                    Matrix::SQUARE_SIZE as u32 - 4,
                    Matrix::SQUARE_SIZE as u32 - 4,
                ))
                .unwrap();
            canvas.set_draw_color(Color::RGB(255, 100, 100));
            canvas
                .draw_rect(Rect::new(
                    self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    self.y + (cell.y as i32) * Matrix::SQUARE_SIZE,
                    Matrix::SQUARE_SIZE as u32,
                    Matrix::SQUARE_SIZE as u32,
                ))
                .unwrap();
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas
            .draw_rect(Rect::new(
                self.x,
                self.y,
                (Board::WIDTH as i32 * Matrix::SQUARE_SIZE) as u32,
                (Board::HEIGHT as i32 * Matrix::SQUARE_SIZE) as u32,
            ))
            .unwrap();
    }
}
pub struct Interface {
    state: GameState,
}

impl Interface {
    pub fn new() -> Self {
        Interface {
            state: GameState::TitleScreen,
        }
    }

    fn handle_input(&mut self, engine: &mut Engine, keycode: Keycode) {
        match self.state {
            GameState::TitleScreen => match keycode {
                Keycode::Space => self.state = GameState::Playing,
                _ => (),
            },
            GameState::Playing => match keycode {
                Keycode::Up => engine.try_move(Direction::CCW),
                Keycode::Down => engine.try_move(Direction::CW),
                Keycode::Left => engine.try_move(Direction::LEFT),
                Keycode::Right => engine.try_move(Direction::RIGHT),
                Keycode::Space => {
                    if let Err(_) = engine.drop() {
                        self.state = GameState::GameOver;
                    }
                }
                Keycode::P => self.state = GameState::Paused,
                _ => (),
            },
            GameState::Paused => match keycode {
                Keycode::P => self.state = GameState::Playing,
                _ => (),
            },
            GameState::GameOver => match keycode {
                Keycode::Space => {
                    engine.clear_board();
                    engine.place_cursor();
                    self.state = GameState::Playing
                }
                _ => (),
            },
        }
    }

    fn draw_queue(&self, canvas: &mut WindowCanvas, engine: &Engine) {
        let start_position: Coordinate = Coordinate::new(310, 20);
        for i in 0..7 {
            let position: Coordinate =
                start_position + Coordinate::new(0, (3 * Matrix::SQUARE_SIZE * i as i32) as isize);
            let kind = engine.queue[i];
            let piece = Piece {
                kind,
                position: Coordinate::new(0, 0),
                rotation: Rotation::N,
            };
            for mino in piece.get_cells() {
                if mino.y < 0 {
                    continue;
                }
                canvas.set_draw_color(Color::RGB(150, 200, 150));
                canvas
                    .fill_rect(Rect::new(
                        position.x as i32 + (2 + mino.x as i32 * Matrix::SQUARE_SIZE),
                        position.y as i32 + (2 + mino.y as i32 * Matrix::SQUARE_SIZE),
                        Matrix::SQUARE_SIZE as u32 - 4,
                        Matrix::SQUARE_SIZE as u32 - 4,
                    ))
                    .unwrap();
                canvas.set_draw_color(Color::RGB(200, 200, 200));
                canvas
                    .draw_rect(Rect::new(
                        position.x as i32 + (mino.x as i32 * Matrix::SQUARE_SIZE),
                        position.y as i32 + (mino.y as i32 * Matrix::SQUARE_SIZE),
                        Matrix::SQUARE_SIZE as u32,
                        Matrix::SQUARE_SIZE as u32,
                    ))
                    .unwrap();
            }
        }
    }

    fn draw_title(&self, msg: &str, canvas: &mut WindowCanvas, font: &mut Font) {
        let texture_creator = canvas.texture_creator();
        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(msg)
            .blended(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let TextureQuery { width, height, .. } = texture.query();

        canvas
            .copy(&texture, None, Some(Rect::new(20, 200, width, height)))
            .map_err(|err| println!("{:?}", err))
            .unwrap();
    }

    pub fn run(&mut self, engine: &mut Engine) {
        engine.place_cursor();
        let matrix = Matrix::new(300, 20);
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

        let window = video_subsystem
            .window("Tetris", 430, 600)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        let mut font_title = ttf_context.load_font("./PixeloidMono.ttf", 45).unwrap();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();
        'running: loop {
            canvas.set_draw_color(Color::RGB(128, 128, 255));
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => self.handle_input(engine, keycode),
                    _ => {}
                }
            }
            // The rest of the game loop goes here...
            match self.state {
                GameState::TitleScreen => self.draw_title("Tetris", &mut canvas, &mut font_title),
                GameState::Playing => {
                    match engine.tick() {
                        Err(e) => {
                            println!("GAMEOVERTICK {:?}", e);
                            self.state = GameState::GameOver
                        }
                        Ok(()) => (),
                    }
                    matrix.draw(&mut canvas, &engine);
                    self.draw_queue(&mut canvas, &engine);
                }
                GameState::Paused => {
                    matrix.draw(&mut canvas, &engine);
                    self.draw_queue(&mut canvas, &engine);
                    self.draw_title(">PAUSE<", &mut canvas, &mut font_title)
                }
                GameState::GameOver => {
                    self.draw_title("GAME OVER. :(", &mut canvas, &mut font_title)
                }
            }

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
