mod piece;

use cgmath::Vector2;
use rand::{prelude::SliceRandom, prelude::ThreadRng, thread_rng};

use self::piece::{Kind, Piece, Rotation};

pub type Coordinate = Vector2<isize>;

struct Board([bool; Self::SIZE]);

impl Board {
    const WIDTH: isize = 10;
    const HEIGHT: isize = 20;
    const SIZE: usize = (Self::WIDTH * Self::HEIGHT) as usize;

    fn blank() -> Self {
        Self([false; Self::SIZE])
    }
}

pub struct Engine {
    board: Board,
    bag: Vec<Kind>,
    rng: ThreadRng,
    pub cursor: Option<Piece>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            board: Board::blank(),
            bag: Vec::new(),
            rng: thread_rng(),
            cursor: None,
        }
    }

    fn fill_bag(&mut self) {
        debug_assert!(self.bag.is_empty());
        self.bag.extend_from_slice(Kind::ALL.as_slice());
        self.bag.shuffle(&mut self.rng);
    }

    pub fn next_piece(&mut self) -> Kind {
        if self.bag.is_empty() {
            self.fill_bag()
        }
        self.bag.pop().unwrap()
    }

    pub fn place_cursor(&mut self) {
        self.cursor = Some(Piece {
            kind: self.next_piece(),
            // NB: We start OFF SCREEN!
            position: Vector2::new(Board::WIDTH as isize / 2, -2),
            rotation: Rotation::N,
        });
    }
}
