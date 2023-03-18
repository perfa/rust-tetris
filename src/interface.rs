extern crate sdl2;

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
        let piece = engine.cursor.as_mut().unwrap();
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
                    } => piece.cw(),
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => piece.ccw(),
                    _ => {}
                }
            }
            let now = Instant::now();
            if now - last_tick > Duration::from_secs(1) {
                piece.lower();
                last_tick = now;
            }
            // The rest of the game loop goes here...
            let minos = piece.get_cells();
            for mino in &minos {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas
                    .draw_rect(Rect::new(
                        (mino.x * 10) as i32,
                        ((mino.y + 2) * 10) as i32,
                        10 as u32,
                        10 as u32,
                    ))
                    .unwrap();
            }
            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
        for _ in 0..4 {
            let minos = piece.get_cells();
            for i in (-2 as isize)..2 {
                for j in (4 as isize)..9 {
                    let mut p: bool = false;
                    for mino in &minos {
                        if mino.x == j as isize && mino.y == i as isize {
                            print!("X");
                            p = true;
                        }
                    }
                    if !p {
                        print!(".");
                    }
                }
                println!("");
            }
            println!("");
            piece.cw();
        }
    }
}
