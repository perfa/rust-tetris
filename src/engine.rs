pub mod piece;

use cgmath::Vector2;
use rand::{prelude::SliceRandom, prelude::ThreadRng, thread_rng};

use self::piece::{Direction, Kind, Piece, Rotation};

pub type Coordinate = Vector2<isize>;

pub struct Board([bool; Self::SIZE as usize]);

impl Board {
    pub const WIDTH: isize = 10;
    pub const HEIGHT: isize = 20;
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

    fn add(&mut self, piece: &Piece) -> Result<(), String> {
        for cell in piece.get_cells() {
            let offset = cell.y * Board::WIDTH + cell.x;
            if offset < 0 {
                return Err("Integer underflow".to_string());
            }
            self.0[offset as usize] = true
        }
        Ok(())
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

    pub fn clear_board(&mut self) {
        self.board = Board::blank();
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
        c.cw(&self.board);
    }

    fn ccw(&mut self) {
        let c = self.cursor.as_mut().unwrap();
        c.ccw(&self.board);
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

    pub fn tick(&mut self) -> Result<(), String> {
        match &self.cursor {
            None => return Result::Ok(()),
            Some(c) => {
                if c.can_lower(&self.board) {
                    self.cursor = Some(c.lower());
                    return Result::Ok(());
                } else {
                    self.board.add(c)?;
                    self.place_cursor();
                }
            }
        }
        if let Some(c) = &self.cursor {
            if !c.can_lower(&self.board) {
                return Result::Err("Game Over".to_string());
            }
        }
        Result::Ok(())
    }

    pub fn drop(&mut self) -> Result<(), String> {
        match &self.cursor {
            None => (),
            Some(c) => {
                let mut p = Piece {
                    kind: c.kind.clone(),
                    position: c.position.clone(),
                    rotation: c.rotation.clone(),
                };
                while p.can_lower(&self.board) {
                    p = p.lower();
                }
                if let Err(_) = self.board.add(&p) {
                    return Err("Game Over".to_string());
                }
                self.place_cursor();
            }
        }
        Ok(())
    }
}
