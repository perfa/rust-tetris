use cgmath::Vector2;
use rand::{
    prelude::{SliceRandom, ThreadRng},
    thread_rng,
};

pub struct Engine {
    board: [bool; 200],
    bag: Vec<Kind>,
    rng: ThreadRng,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            board: [false; 200],
            bag: Vec::new(),
            rng: thread_rng(),
        }
    }

    fn fill_bag(&mut self) {
        self.bag.extend_from_slice(Kind::ALL.as_slice());
        self.bag.shuffle(&mut self.rng);
    }
}

pub struct Piece {
    pub kind: Kind,
    pub position: Vector2<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    O,
    I,
    T,
    L,
    J,
    S,
    Z,
}

impl Kind {
    pub const ALL: [Self; 7] = [
        Self::O,
        Self::I,
        Self::T,
        Self::L,
        Self::J,
        Self::S,
        Self::Z,
    ];
}
