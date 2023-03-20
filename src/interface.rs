extern crate sdl2;

use crate::engine::piece::Direction;
use crate::engine::{Board, Engine};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::{event::Event, render::WindowCanvas};
use std::time::{Duration, Instant};

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
        canvas.set_draw_color(Color::RGB(200, 200, 200));
        let minos = engine.cursor.as_ref().unwrap().get_cells();
        for mino in &minos {
            if mino.y < 0 {
                continue;
            }
            canvas
                .draw_rect(Rect::new(
                    self.x + (mino.x as i32) * Matrix::SQUARE_SIZE,
                    self.y + (mino.y as i32) * Matrix::SQUARE_SIZE,
                    Matrix::SQUARE_SIZE as u32,
                    Matrix::SQUARE_SIZE as u32,
                ))
                .unwrap();
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        for cell in engine.get_pile() {
            canvas
                .draw_rect(Rect::new(
                    self.x + (cell.x as i32) * Matrix::SQUARE_SIZE,
                    self.y + (cell.y as i32) * Matrix::SQUARE_SIZE,
                    Matrix::SQUARE_SIZE as u32,
                    Matrix::SQUARE_SIZE as u32,
                ))
                .unwrap();
        }
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
pub struct Interface {}

impl Interface {
    pub fn run(engine: &mut Engine) {
        engine.place_cursor();
        let matrix = Matrix::new(300, 20);
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo", 300, 600)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut last_tick = Instant::now();
        'running: loop {
            canvas.set_draw_color(Color::RGB(128, 128, 255));
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => engine.try_move(Direction::CCW),
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => engine.try_move(Direction::CW),
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => engine.try_move(Direction::LEFT),
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => engine.try_move(Direction::RIGHT),
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => engine.drop(),
                    _ => {}
                }
            }
            let now = Instant::now();
            if now - last_tick > Duration::from_millis(250) {
                engine.tick();
                last_tick = now;
            }
            // The rest of the game loop goes here...
            matrix.draw(&mut canvas, &engine);
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
