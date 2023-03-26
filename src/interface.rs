extern crate sdl2;

use crate::engine::piece::{Direction, Piece, Rotation};
use crate::engine::{Board, Coordinate, Engine};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::ttf::Font;
use sdl2::EventPump;
use sdl2::{event::Event, render::WindowCanvas};
use std::collections::HashSet;
use std::time::{Duration, Instant};

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

#[derive(Debug, Copy, Clone, PartialEq)]
enum AutoRepeat {
    NoPress,
    Pressed(Scancode, Instant),
    Repeating(Scancode, Instant),
}

pub struct Interface {
    state: GameState,
    pressed_keys: HashSet<Scancode>,
    auto_repeat: AutoRepeat,
}

impl Interface {
    pub fn new() -> Self {
        Interface {
            state: GameState::TitleScreen,
            pressed_keys: HashSet::new(),
            auto_repeat: AutoRepeat::NoPress,
        }
    }

    fn get_scancodes(old: &HashSet<Scancode>, new: &HashSet<Scancode>) -> HashSet<Scancode> {
        new - old
    }

    fn handle_input(&mut self, engine: &mut Engine, event_pump: &mut EventPump) {
        let scancodes: HashSet<Scancode> =
            event_pump.keyboard_state().pressed_scancodes().collect();
        let newly_pressed: HashSet<Scancode> =
            Interface::get_scancodes(&self.pressed_keys, &scancodes);
        self.pressed_keys = scancodes;

        match self.state {
            GameState::TitleScreen => {
                if newly_pressed.contains(&Scancode::Space) {
                    self.state = GameState::Playing
                }
            }
            GameState::Playing => {
                if newly_pressed.contains(&Scancode::P) {
                    self.state = GameState::Paused;
                }
                if newly_pressed.contains(&Scancode::Up) {
                    engine.try_move(Direction::CCW);
                }
                if newly_pressed.contains(&Scancode::Down) {
                    // TODO: Make soft-drop, shift to right ctrl/cmd
                    engine.try_move(Direction::CW);
                }
                if newly_pressed.contains(&Scancode::Space) {
                    if let Err(_) = engine.drop() {
                        self.state = GameState::GameOver;
                    }
                }
                // NB: XOR!
                if newly_pressed.contains(&Scancode::Left)
                    || newly_pressed.contains(&Scancode::Right)
                {
                    let direction;
                    if newly_pressed.contains(&Scancode::Left) {
                        direction = Direction::LEFT;
                        self.auto_repeat = AutoRepeat::Pressed(Scancode::Left, Instant::now());
                    } else {
                        direction = Direction::RIGHT;
                        self.auto_repeat = AutoRepeat::Pressed(Scancode::Right, Instant::now());
                    }

                    engine.try_move(direction)
                }

                let scancodes: HashSet<Scancode> =
                    event_pump.keyboard_state().pressed_scancodes().collect();
                let direction;
                if scancodes.contains(&Scancode::Left) {
                    direction = Direction::LEFT;
                } else {
                    direction = Direction::RIGHT;
                }
                match self.auto_repeat {
                    AutoRepeat::NoPress => (),
                    AutoRepeat::Pressed(scancode, start) => {
                        if !scancodes.contains(&scancode) {
                            self.auto_repeat = AutoRepeat::NoPress;
                            return;
                        }
                        let duration = Instant::now() - start;
                        if duration >= Duration::from_millis(300) {
                            self.auto_repeat = AutoRepeat::Repeating(scancode, Instant::now());
                            engine.try_move(direction);
                        }
                    }
                    AutoRepeat::Repeating(scancode, start) => {
                        if !scancodes.contains(&scancode) {
                            self.auto_repeat = AutoRepeat::NoPress;
                            return;
                        }
                        let duration = Instant::now() - start;
                        if duration >= Duration::from_millis(28) {
                            engine.try_move(direction);
                        }
                    }
                }
            }
            GameState::Paused => {
                if newly_pressed.contains(&Scancode::P) {
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                if newly_pressed.contains(&Scancode::Space) {
                    engine.clear_board();
                    engine.place_cursor();
                    self.state = GameState::Playing
                }
            }
        }
    }

    fn draw_queue(&self, canvas: &mut WindowCanvas, engine: &Engine) {
        let start_position: Coordinate = Coordinate::new(310, 20);
        for i in 0..4 {
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
            let loop_start = Instant::now();
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
                    _ => {}
                }
            }
            self.handle_input(engine, &mut event_pump);
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
            let cycle_time = Instant::now() - loop_start;
            let one_sixieth_second = Duration::new(0, 1_000_000_000u32 / 60);
            if one_sixieth_second > cycle_time {
                let remaining = one_sixieth_second - cycle_time;
                ::std::thread::sleep(remaining);
            }
        }
    }
}
