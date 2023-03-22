#![allow(dead_code)]
mod engine;
mod interface;

fn main() {
    let mut engine = engine::Engine::new();
    let mut if_ = interface::Interface::new();
    if_.run(&mut engine);
}
