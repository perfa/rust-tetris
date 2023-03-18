#![allow(dead_code)]
mod engine;
mod interface;

fn main() {
    let mut engine = engine::Engine::new();
    let _ = interface::Interface::run(&mut engine);
}
