pub mod piece;

use cgmath::Vector2;
use rand::{prelude::SliceRandom, prelude::ThreadRng, thread_rng};
use std::time::{Duration, Instant};

use self::piece::{Direction, Kind, Piece, Rotation};

pub type Coordinate = Vector2<isize>;

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    filled: bool,
    marked: bool,
}

impl Cell {
    fn new() -> Self {
        Cell {
            filled: false,
            marked: false,
        }
    }
}

pub struct Board([Cell; Self::SIZE as usize], Vec<isize>);

impl Board {
    pub const WIDTH: isize = 10;
    pub const HEIGHT: isize = 20;
    const SIZE: isize = (Self::WIDTH * Self::HEIGHT);

    fn blank() -> Self {
        Self([Cell::new(); Self::SIZE as usize], vec![])
    }

    fn filled(&self, coord: Coordinate) -> bool {
        if coord.y < 0 {
            return false;
        }

        let offset = coord.y * Board::WIDTH + coord.x;
        if offset >= Board::SIZE {
            return true;
        }

        self.0[offset as usize].filled
    }

    fn add(&mut self, piece: &Piece) -> Result<(), String> {
        for cell in piece.get_cells() {
            let offset = cell.y * Board::WIDTH + cell.x;
            if offset < 0 {
                return Err("Integer underflow".to_string());
            }
            self.0[offset as usize].filled = true;
        }
        Ok(())
    }

    fn has_patterns(&mut self) -> bool {
        let mut found = false;
        for row in 0..Board::HEIGHT {
            let mut cells: Vec<usize> = vec![];
            for col in 0..Board::WIDTH {
                let offset = (row * Board::WIDTH + col) as usize;
                if self.0[offset].filled {
                    cells.push(offset);
                }
            }
            if cells.len() == Board::WIDTH as usize {
                found = true;
                cells.into_iter().for_each(|c| self.0[c].marked = true);
                self.1.push(row);
            }
        }
        found
    }

    fn clear_marked(&mut self) -> bool {
        self.1.sort();
        self.1.reverse();
        let mut moved_cell = false;

        match self.1.pop() {
            Some(cleared_row) => {
                for col in 0..Board::WIDTH {
                    let offset = (cleared_row * Board::WIDTH + col) as usize;
                    self.0[offset] = Cell::new();
                }
                // Checking from above the empty row upwards, filling down is safe
                for row in (0..cleared_row).rev() {
                    for col in 0..Board::WIDTH {
                        let offset = (row * Board::WIDTH + col) as usize;
                        let offset_below = ((row + 1) * Board::WIDTH + col) as usize;
                        if self.0[offset].filled {
                            moved_cell = true;
                            self.0[offset] = Cell::new();
                            self.0[offset_below].filled = true;
                        }
                    }
                }
            }
            None => return false,
        }
        moved_cell
    }

    fn lower_floaters(&mut self) -> bool {
        let mut fallers: Vec<usize> = vec![];

        //We have the rows that are cleared in self.1

        for row in 0..Board::HEIGHT {
            for col in 0..Board::WIDTH {
                let offset = (row * Board::WIDTH + col) as usize;

                if self.0[offset].marked {
                    fallers.push(offset);
                }
            }
        }
        true // They are all lowered
    }
}

#[derive(Debug)]
pub enum EngineState {
    Falling,
    Locking(Instant),
    PatternFinding,
    Animating(Instant),
    EliminatingSpace,
    Completing,
}

pub struct Engine {
    board: Board,
    bag: Vec<Kind>,
    rng: ThreadRng,
    last_tick: Instant,
    pub state: EngineState,
    pub cursor: Option<Piece>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            board: Board::blank(),
            bag: Vec::new(),
            rng: thread_rng(),
            last_tick: Instant::now(),
            state: EngineState::Falling,
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

    fn next_piece_from_bag(&mut self) -> Kind {
        if self.bag.is_empty() {
            self.fill_bag()
        }
        self.bag.pop().unwrap()
    }

    pub fn place_cursor(&mut self) {
        self.cursor = Some(Piece {
            kind: self.next_piece_from_bag(),
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
        if let Some(c) = self.cursor.as_mut() {
            if c.can_move_lateral(&self.board, direction) {
                c.lateral_move(direction)
            }
        }
    }

    fn cw(&mut self) {
        if let Some(c) = self.cursor.as_mut() {
            c.cw(&self.board);
        }
    }

    fn ccw(&mut self) {
        if let Some(c) = self.cursor.as_mut() {
            c.ccw(&self.board);
        }
    }

    pub fn get_pile(&self) -> Vec<Coordinate> {
        let mut cells: Vec<Coordinate> = vec![];
        for offset in 0..Board::SIZE {
            if self.board.0[offset as usize].filled & !self.board.0[offset as usize].marked {
                cells.push(Coordinate {
                    x: offset % Board::WIDTH,
                    y: offset / Board::WIDTH,
                })
            }
        }
        cells
    }

    pub fn get_marked(&self) -> Vec<Coordinate> {
        let mut cells: Vec<Coordinate> = vec![];
        for offset in 0..Board::SIZE {
            if self.board.0[offset as usize].filled & self.board.0[offset as usize].marked {
                cells.push(Coordinate {
                    x: offset % Board::WIDTH,
                    y: offset / Board::WIDTH,
                })
            }
        }
        cells
    }

    pub fn tick(&mut self) -> Result<(), String> {
        println!("State: {:?}", self.state);
        match self.state {
            EngineState::Falling => match &self.cursor {
                None => {
                    self.place_cursor();
                    if let Some(c) = &self.cursor {
                        if !c.can_lower(&self.board) {
                            return Result::Err("Cannot lower new cursor".to_string());
                        }
                    }
                }
                Some(c) => {
                    let now = Instant::now();
                    if now - self.last_tick > Duration::from_millis(250) {
                        if c.can_lower(&self.board) {
                            self.cursor = Some(c.lower());
                            self.last_tick = now;
                            return Result::Ok(());
                        } else {
                            self.state = EngineState::Locking(Instant::now());
                            return Result::Ok(());
                        }
                    }
                }
            },
            EngineState::Locking(start) => {
                if let Some(c) = &self.cursor {
                    if c.can_lower(&self.board) {
                        self.cursor = Some(c.lower());
                        self.state = EngineState::Falling;
                        return Result::Ok(());
                    }
                }
                if (Instant::now() - start) > Duration::from_millis(500) {
                    if let Some(c) = &self.cursor {
                        self.board.add(c)?;
                        self.cursor = None;
                    }
                    self.state = EngineState::PatternFinding;
                }
            }
            EngineState::PatternFinding => match self.board.has_patterns() {
                true => self.state = EngineState::Animating(Instant::now()),
                false => self.state = EngineState::Falling,
            },
            EngineState::Animating(start) => {
                if (Instant::now() - start) > Duration::from_millis(500) {
                    if self.board.clear_marked() {
                        self.state = EngineState::EliminatingSpace;
                    } else {
                        self.state = EngineState::Falling;
                    }
                }
            }
            EngineState::EliminatingSpace => {
                if self.board.lower_floaters() {
                    self.state = EngineState::Animating(Instant::now());
                }
            }
            EngineState::Completing => todo!(), // Score, level up, etc
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
                self.cursor = None;
                self.state = EngineState::PatternFinding;
            }
        }
        Ok(())
    }
}
