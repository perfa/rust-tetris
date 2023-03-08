#![allow(dead_code)]
mod engine;
mod interface;

fn main() {
    let _engine = engine::Engine::new();
    let _ = interface::Interface::run();
    println!("Hello, world!");
}
