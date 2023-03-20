pub mod piece;

use cgmath::Vector2;
use rand::{prelude::SliceRandom, prelude::ThreadRng, thread_rng};

use self::piece::{Direction, Kind, Piece, Rotation};

pub type Coordinate = Vector2<isize>;

pub struct Board([bool; Self::SIZE as usize]);

impl Board {
    const WIDTH: isize = 10;
    const HEIGHT: isize = 20;
    const SIZE: isize = (Self::WIDTH * Self::HEIGHT);

    fn blank() -> Self {
        Self([false; Self::SIZE as usize])
    }

    fn filled(&self, coord: Coordinate) -> bool {
        if coord.y < 0 {
            return false;
        }

        let offset = coord.y * Board::WIDTH + coord.x;
        if offset >= Board::SIZE {
            return true;
        }

        self.0[offset as usize]
    }

    fn add(&mut self, piece: &Piece) {
        for cell in piece.get_cells() {
            let offset = (cell.y * Board::WIDTH + cell.x) as usize;
            self.0[offset] = true
        }
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

    pub fn try_move(&mut self, direction: Direction) {
        match direction {
            Direction::LEFT => self.left_or_right(direction),
            Direction::RIGHT => self.left_or_right(direction),
            Direction::CW => self.cw(),
            Direction::CCW => self.ccw(),
        }
    }

    fn left_or_right(&mut self, direction: Direction) {
        let c = self.cursor.as_mut().unwrap();
        if c.can_move_lateral(&self.board, direction) {
            c.lateral_move(direction)
        }
    }

    fn cw(&mut self) {
        let c = self.cursor.as_mut().unwrap();
        c.cw();
    }

    fn ccw(&mut self) {
        let c = self.cursor.as_mut().unwrap();
        c.ccw();
    }

    pub fn get_pile(&self) -> Vec<Coordinate> {
        let mut cells: Vec<Coordinate> = vec![];
        for offset in 0..Board::SIZE {
            if self.board.0[offset as usize] {
                cells.push(Coordinate {
                    x: offset % Board::WIDTH,
                    y: offset / Board::WIDTH,
                })
            }
        }
        cells
    }

    pub fn tick(&mut self) {
        match &self.cursor {
            None => (),
            Some(c) => {
                if c.can_lower(&self.board) {
                    self.cursor = Some(c.lower());
                } else {
                    self.board.add(c);
                    self.place_cursor();
                }
            }
        }
    }
}
