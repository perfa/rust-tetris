pub mod piece;

use cgmath::Vector2;
use rand::{prelude::SliceRandom, prelude::ThreadRng, thread_rng};
use std::{
    cmp,
    collections::VecDeque,
    time::{Duration, Instant},
};

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
            let offset = (cell.y + 1) * Board::WIDTH + cell.x;
            if offset < 0 {
                return Err("Integer underflow".to_string());
            }
            self.0[offset as usize].filled = true;
        }
        Ok(())
    }

    fn has_patterns(&mut self, points: &mut usize, level: &mut usize) -> bool {
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
        match self.1.len() {
            1 => *points += 100 * *level,
            2 => *points += 300 * *level,
            3 => *points += 500 * *level,
            4 => *points += 800 * *level,
            _ => (), // Can't happen
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
}

#[derive(Debug)]
pub enum EngineState {
    Falling,
    Locking(Instant),
    PatternFinding,
    Animating(Instant),
    EliminatingSpace,
}

pub struct Engine {
    board: Board,
    bag: Vec<Kind>,
    rng: ThreadRng,
    last_tick: Instant,
    soft_dropping: bool,
    soft_drop_count: usize,
    pub level: usize,
    pub rows_cleared: usize,
    pub points: usize,
    pub state: EngineState,
    pub queue: VecDeque<Kind>,
    pub cursor: Option<Piece>,
}

impl Engine {
    const LEVEL_TPR_IN_MS: [u32; 15] = [
        1000, 793, 618, 473, 355, 262, 190, 135, 94, 64, 43, 28, 18, 11, 7,
    ];
    pub fn new() -> Self {
        Engine {
            board: Board::blank(),
            bag: Vec::new(),
            rng: thread_rng(),
            level: 1,
            rows_cleared: 0,
            points: 0,
            soft_dropping: false,
            soft_drop_count: 0,
            last_tick: Instant::now(),
            state: EngineState::Falling,
            queue: VecDeque::with_capacity(7),
            cursor: None,
        }
    }

    fn fill_bag(&mut self) {
        debug_assert!(self.bag.is_empty());
        self.bag.extend_from_slice(Kind::ALL.as_slice());
        self.bag.shuffle(&mut self.rng);
    }

    fn fill_queue(&mut self) {
        for _ in 0..7 {
            let kind = self.next_piece_from_bag();
            self.queue.push_back(kind);
        }
    }

    fn pull_from_queue(&mut self) -> Kind {
        if self.queue.is_empty() {
            self.fill_queue();
        }
        let kind = self.queue.pop_front().unwrap();
        let new = self.next_piece_from_bag();
        self.queue.push_back(new);
        kind
    }

    pub fn clear_board(&mut self) {
        self.points = 0;
        self.rows_cleared = 0;
        self.level = 1;
        self.soft_dropping = false;
        self.soft_drop_count = 0;
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
            kind: self.pull_from_queue(),
            current_position: Coordinate::new((Board::WIDTH as isize / 2) - 2, -2),
            offset: 0.0,
            // NB: We start OFF SCREEN!
            position: Vector2::new((Board::WIDTH as isize / 2) - 2, -2),
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

    pub fn tick(&mut self, soft_drop: bool) -> Result<(), String> {
        // println!("State: {:?}", self.state);
        let stopped_soft_dropping = if self.soft_dropping && !soft_drop {
            self.points += self.level * self.soft_drop_count;
            self.soft_drop_count = 0;
            true
        } else {
            false
        };
        self.soft_dropping = soft_drop;
        match self.state {
            EngineState::Falling => match &mut self.cursor {
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
                    let duration_divisor = if soft_drop || stopped_soft_dropping {
                        20
                    } else {
                        1
                    };
                    let level_tick_duration =
                        Self::LEVEL_TPR_IN_MS[self.level - 1] / duration_divisor;
                    let elapsed = now - self.last_tick;
                    let percentage = cmp::min(level_tick_duration, elapsed.as_millis() as u32)
                        as f32
                        / level_tick_duration as f32;
                    c.offset = percentage;
                    if stopped_soft_dropping {
                        let new_level_tick_duration = level_tick_duration * 20;
                        /* old duration, old percentage
                        new duration, SAME percentage --> self.tick needs to
                        move backwards in time.
                        new_last_tick + percentage * new_duration = now()
                        */
                        self.last_tick = now
                            - Duration::from_millis(
                                (percentage * new_level_tick_duration as f32) as u64,
                            );
                        return Result::Ok(());
                    }
                    if elapsed > Duration::from_millis(level_tick_duration as u64) {
                        if c.can_lower(&self.board) {
                            if self.soft_dropping {
                                self.soft_drop_count += 1;
                            }
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
                        if self.soft_dropping {
                            self.points += self.level * self.soft_drop_count;
                            self.soft_drop_count = 0;
                        }
                        self.board.add(c)?;
                        self.cursor = None;
                    }
                    self.state = EngineState::PatternFinding;
                }
            }
            EngineState::PatternFinding => {
                match self.board.has_patterns(&mut self.points, &mut self.level) {
                    true => self.state = EngineState::Animating(Instant::now()),
                    false => self.state = EngineState::Falling,
                }
            }
            EngineState::Animating(start) => {
                if (Instant::now() - start) > Duration::from_millis(100) {
                    if self.board.clear_marked() {
                        self.rows_cleared += 1;
                        if self.rows_cleared >= (self.level * 10) {
                            self.level += 1;
                        }
                        self.state = EngineState::EliminatingSpace;
                    } else {
                        self.state = EngineState::Falling;
                    }
                }
            }
            EngineState::EliminatingSpace => {
                // Reset animation timer, "eliminating space" is in
                // the drawing code more concretely speaking.
                self.state = EngineState::Animating(Instant::now());
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
                    current_position: c.position.clone(),
                    offset: c.offset,
                    position: c.position.clone(),
                    rotation: c.rotation.clone(),
                };
                let mut drop_height = 0;
                while p.can_lower(&self.board) {
                    drop_height += 1;
                    p = p.lower();
                }
                self.points += 2 * drop_height;
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
