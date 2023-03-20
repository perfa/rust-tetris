extern crate sdl2;

use crate::engine::piece::Direction;
use crate::engine::Engine;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};

pub struct Interface {}

impl Interface {
    pub fn run(engine: &mut Engine) {
        engine.place_cursor();
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo", 800, 600)
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
                    _ => {}
                }
            }
            let now = Instant::now();
            if now - last_tick > Duration::from_millis(250) {
                engine.tick();
                last_tick = now;
            }
            // The rest of the game loop goes here...
            canvas.set_draw_color(Color::RGB(200, 200, 200));
            let minos = engine.cursor.as_mut().unwrap().get_cells();
            for mino in &minos {
                canvas
                    .draw_rect(Rect::new(
                        (mino.x * 10) as i32,
                        ((mino.y + 2) * 10) as i32,
                        10 as u32,
                        10 as u32,
                    ))
                    .unwrap();
            }

            canvas.set_draw_color(Color::RGB(0, 0, 0));
            for cell in engine.get_pile() {
                canvas
                    .draw_rect(Rect::new(
                        (cell.x * 10) as i32,
                        ((cell.y + 2) * 10) as i32,
                        10 as u32,
                        10 as u32,
                    ))
                    .unwrap();
            }
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
