extern crate sdl2;

use crate::engine::piece::{Direction, Kind, Piece, Rotation};
use crate::engine::{Board, Coordinate, Engine};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::ttf::Font;
use sdl2::EventPump;
use sdl2::{event::Event, render::WindowCanvas};
use std::cmp;
use std::collections::HashSet;
use std::time::{Duration, Instant};

enum GameState {
    TitleScreen,
    Playing,
    Paused,
    GameOver,
}

#[non_exhaustive]
struct Colors;

impl Colors {
    pub const BG: Color = Color::RGB(128, 128, 255);
    pub const LIVE_AREA: Color = Color::RGB(187, 183, 190);
    pub const LIVE_CELL: Color = Color::RGB(150, 200, 150);
    pub const LOCKED_CELL: Color = Color::RGB(100, 150, 100);
    pub const MARKED_CELL: Color = Color::RGB(255, 100, 100);
}

struct PieceQueue {
    x: i32,
    y: i32,
    shown_items: usize,
}

impl PieceQueue {
    fn draw(&self, canvas: &mut WindowCanvas, engine: &Engine) {
        canvas.set_draw_color(Colors::LIVE_AREA);
        canvas
            .fill_rect(Rect::new(
                self.x - Matrix::SQUARE_SIZE,
                self.y,
                (5 * Matrix::SQUARE_SIZE) as u32,
                (self.shown_items * 3 * Matrix::SQUARE_SIZE as usize) as u32,
            ))
            .unwrap();

        canvas.set_draw_color(Color::BLACK);
        canvas
            .draw_rect(Rect::new(
                self.x - Matrix::SQUARE_SIZE,
                self.y,
                (5 * Matrix::SQUARE_SIZE) as u32,
                (self.shown_items * 3 * Matrix::SQUARE_SIZE as usize) as u32,
            ))
            .unwrap();

        let start_position: Coordinate = Coordinate::new(self.x as isize, self.y as isize + 10);
        for i in 0..self.shown_items {
            let position: Coordinate =
                start_position + Coordinate::new(0, (3 * Matrix::SQUARE_SIZE * i as i32) as isize);
            let kind = engine.queue[i];
            let x_adjust = if [Kind::I, Kind::O].contains(&kind) {
                Matrix::SQUARE_SIZE / 2
            } else {
                0
            };
            let piece = Piece {
                kind,
                current_position: Coordinate::new(0, 0),
                offset: 0.0,
                position: Coordinate::new(0, 0),
                rotation: Rotation::N,
            };
            for mino in piece.get_cells() {
                if mino.y < 0 {
                    continue;
                }
                canvas.set_draw_color(Colors::LIVE_CELL);
                canvas
                    .fill_rect(Rect::new(
                        position.x as i32 + (2 + mino.x as i32 * Matrix::SQUARE_SIZE) - x_adjust,
                        position.y as i32 + (2 + mino.y as i32 * Matrix::SQUARE_SIZE),
                        Matrix::SQUARE_SIZE as u32 - 4,
                        Matrix::SQUARE_SIZE as u32 - 4,
                    ))
                    .unwrap();
                canvas.set_draw_color(Color::BLACK);
                canvas
                    .draw_rect(Rect::new(
                        position.x as i32 + (mino.x as i32 * Matrix::SQUARE_SIZE) - x_adjust,
                        position.y as i32 + (mino.y as i32 * Matrix::SQUARE_SIZE),
                        Matrix::SQUARE_SIZE as u32,
                        Matrix::SQUARE_SIZE as u32,
                    ))
                    .unwrap();
            }
        }
    }

    fn new(left_offset: i32, top_offset: i32) -> Self {
        PieceQueue {
            x: left_offset,
            y: top_offset,
            shown_items: 4,
        }
    }
}

struct Matrix {
    x: i32,
    y: i32,
}

impl Matrix {
    const SQUARE_SIZE: i32 = 27;
    const ONE_THIRD: i32 = 9;
    const TWO_THIRDS: i32 = 18;
    pub fn new(left_offset: i32, top_offset: i32) -> Self {
        Matrix {
            x: left_offset,
            y: top_offset,
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas, engine: &Engine) {
        canvas.set_draw_color(Colors::LIVE_AREA);
        canvas
            .fill_rect(Rect::new(
                self.x,
                self.y,
                (Board::WIDTH as i32 * Matrix::SQUARE_SIZE) as u32,
                ((Board::HEIGHT as i32 * Matrix::SQUARE_SIZE) + Matrix::ONE_THIRD) as u32,
            ))
            .unwrap();

        if let Some(cursor) = engine.cursor.as_ref() {
            let minos = cursor.get_cells();
            let pixel_offset_y = ((cursor.position.y - cursor.current_position.y) as f32 // # of squares
                * cursor.offset  // % completed
                * Matrix::SQUARE_SIZE as f32) as i32; // pixels/whole square
            debug_assert!(pixel_offset_y <= Matrix::SQUARE_SIZE);
            for mino in &minos {
                if mino.y < -1 && pixel_offset_y < Matrix::TWO_THIRDS {
                    continue;
                }

                let height;
                let x = self.x + (mino.x as i32) * Matrix::SQUARE_SIZE;
                let y;
                if mino.y <= -1 {
                    let mino_offset = (mino.y - -1) as i32 * Matrix::SQUARE_SIZE; // 0 or 1 squares
                    y = cmp::max(
                        self.y as i32,
                        self.y + (pixel_offset_y as i32 - Matrix::TWO_THIRDS) + mino_offset,
                    );
                    height = cmp::min(
                        Matrix::ONE_THIRD + pixel_offset_y + mino_offset,
                        Matrix::SQUARE_SIZE,
                    );
                } else {
                    y = self.y
                        + pixel_offset_y
                        + (mino.y as i32) * Matrix::SQUARE_SIZE
                        + Matrix::SQUARE_SIZE / 3;
                    height = Matrix::SQUARE_SIZE;
                };
                canvas.set_draw_color(Colors::LIVE_CELL);
                canvas
                    .fill_rect(Rect::new(
                        x + 2,
                        y + 2,
                        Matrix::SQUARE_SIZE as u32 - 4,
                        cmp::max(height, 4) as u32 - 4,
                    ))
                    .unwrap();
                canvas.set_draw_color(Color::RGB(20, 20, 20));
                canvas
                    .draw_rect(Rect::new(x, y, Matrix::SQUARE_SIZE as u32, height as u32))
                    .unwrap();
            }
        }

        for cell in engine.get_pile() {
            canvas.set_draw_color(Colors::LOCKED_CELL);
            canvas
                .fill_rect(Rect::new(
                    2 + self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    2 + self.y + (cell.y as i32) * Matrix::SQUARE_SIZE + Matrix::SQUARE_SIZE / 3,
                    Matrix::SQUARE_SIZE as u32 - 4,
                    Matrix::SQUARE_SIZE as u32 - 4,
                ))
                .unwrap();
            canvas.set_draw_color(Color::BLACK);
            canvas
                .draw_rect(Rect::new(
                    self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    self.y + (cell.y as i32) * Matrix::SQUARE_SIZE + Matrix::SQUARE_SIZE / 3,
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
                    2 + self.y + (cell.y as i32) * Matrix::SQUARE_SIZE + Matrix::SQUARE_SIZE / 3,
                    Matrix::SQUARE_SIZE as u32 - 4,
                    Matrix::SQUARE_SIZE as u32 - 4,
                ))
                .unwrap();
            canvas.set_draw_color(Colors::MARKED_CELL);
            canvas
                .draw_rect(Rect::new(
                    self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    self.y + (cell.y as i32) * Matrix::SQUARE_SIZE + Matrix::SQUARE_SIZE / 3,
                    Matrix::SQUARE_SIZE as u32,
                    Matrix::SQUARE_SIZE as u32,
                ))
                .unwrap();
        }

        canvas.set_draw_color(Color::BLACK);
        canvas
            .draw_rect(Rect::new(
                self.x,
                self.y,
                (Board::WIDTH as i32 * Matrix::SQUARE_SIZE) as u32,
                ((Board::HEIGHT as i32 * Matrix::SQUARE_SIZE) + (Matrix::SQUARE_SIZE / 3)) as u32,
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
    soft_drop: bool,
}

impl Interface {
    pub fn new() -> Self {
        Interface {
            state: GameState::TitleScreen,
            pressed_keys: HashSet::new(),
            auto_repeat: AutoRepeat::NoPress,
            soft_drop: false,
        }
    }

    fn get_scancodes(old: &HashSet<Scancode>, new: &HashSet<Scancode>) -> HashSet<Scancode> {
        new - old
    }

    fn handle_input(&mut self, engine: &mut Engine, event_pump: &mut EventPump) {
        let scancodes: HashSet<Scancode> =
            event_pump.keyboard_state().pressed_scancodes().collect();

        self.soft_drop = scancodes.contains(&Scancode::Down);

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
                if newly_pressed.contains(&Scancode::RCtrl) {
                    engine.try_move(Direction::CW);
                }
                if newly_pressed.contains(&Scancode::Space) {
                    if let Err(_) = engine.drop() {
                        self.state = GameState::GameOver;
                    }
                }
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

    fn draw_text(
        &self,
        msg: &str,
        canvas: &mut WindowCanvas,
        font: &mut Font,
        color: Color,
        x: u32,
        y: u32,
        centered: bool,
    ) {
        let texture_creator = canvas.texture_creator();

        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(msg)
            .blended(color)
            .map_err(|e| e.to_string())
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let TextureQuery { width, height, .. } = texture.query();

        let x_pos;
        let y_pos;
        if centered {
            let (canvas_width, canvas_height) = canvas.viewport().size();
            x_pos = if width > canvas_width {
                0
            } else {
                (canvas_width - width) / 2
            };
            y_pos = if (height + y) > canvas_height {
                y
            } else {
                (canvas_height - height) / 2 + y
            };
        } else {
            x_pos = x;
            y_pos = y;
        }
        canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(x_pos as i32, y_pos as i32, width, height)),
            )
            .map_err(|err| println!("{:?}", err))
            .unwrap();
    }

    fn draw_title(
        &self,
        msg: &str,
        canvas: &mut WindowCanvas,
        font: &mut Font,
        offset: Option<u32>,
    ) {
        let y_offset = offset.unwrap_or(0);
        self.draw_text(msg, canvas, font, Color::RED, 0, y_offset, true)
    }

    fn draw_stats(&self, canvas: &mut WindowCanvas, engine: &Engine, font: &mut Font) {
        let spacing: u32 = font.height() as u32;
        self.draw_text(&"Level", canvas, font, Color::BLACK, 10, 20, false);
        self.draw_text(
            format!("{}", engine.level).as_str(),
            canvas,
            font,
            Color::RED,
            10,
            20 + spacing,
            false,
        );

        self.draw_text(
            &"Score",
            canvas,
            font,
            Color::BLACK,
            10,
            20 + (spacing * 3),
            false,
        );
        self.draw_text(
            format!("{}", engine.points).as_str(),
            canvas,
            font,
            Color::RED,
            10,
            20 + (spacing * 4),
            false,
        );

        self.draw_text(
            &"Cleared",
            canvas,
            font,
            Color::BLACK,
            10,
            20 + (spacing * 6),
            false,
        );
        self.draw_text(
            format!("{}", engine.rows_cleared).as_str(),
            canvas,
            font,
            Color::RED,
            10,
            20 + (spacing * 7),
            false,
        );
    }

    pub fn run(&mut self, engine: &mut Engine) {
        engine.place_cursor();
        let matrix = Matrix::new(120, 20);
        let mut queue = PieceQueue::new(460, 20);
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

        let window = video_subsystem
            .window("Tetris", 580, 600)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        let mut font_title = ttf_context.load_font("./PixeloidMono.ttf", 45).unwrap();
        let mut font_stats = ttf_context.load_font("./PixeloidMono.ttf", 12).unwrap();

        canvas.set_draw_color(Colors::BG);
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();
        'running: loop {
            let loop_start = Instant::now();
            canvas.set_draw_color(Colors::BG);
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
                        keycode: Some(Keycode::Period),
                        ..
                    } => queue.shown_items = cmp::min(7, queue.shown_items + 1),
                    Event::KeyDown {
                        keycode: Some(Keycode::Comma),
                        ..
                    } => queue.shown_items = cmp::max(1, queue.shown_items - 1),
                    _ => {}
                }
            }
            self.handle_input(engine, &mut event_pump);
            match self.state {
                GameState::TitleScreen => {
                    self.draw_title("Tetris", &mut canvas, &mut font_title, None);
                    self.draw_title(
                        ">PRESS SPACE TO START<",
                        &mut canvas,
                        &mut font_stats,
                        Some(60),
                    );
                }
                GameState::Playing => {
                    match engine.tick(self.soft_drop) {
                        Err(e) => {
                            println!("GAMEOVERTICK {:?}", e);
                            self.state = GameState::GameOver
                        }
                        Ok(()) => (),
                    }
                    matrix.draw(&mut canvas, &engine);
                    self.draw_stats(&mut canvas, &engine, &mut font_stats);
                    queue.draw(&mut canvas, &engine);
                }
                GameState::Paused => {
                    matrix.draw(&mut canvas, &engine);
                    self.draw_stats(&mut canvas, &engine, &mut font_stats);
                    queue.draw(&mut canvas, &engine);
                    self.draw_title(">PAUSE<", &mut canvas, &mut font_title, None)
                }
                GameState::GameOver => {
                    self.draw_title("GAME OVER. :(", &mut canvas, &mut font_title, None)
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
