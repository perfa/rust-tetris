use std::collections::HashMap;

use lazy_static::lazy_static;

use super::Coordinate;

lazy_static! {
    static ref SHAPES: HashMap<&'static str, HashMap<&'static str, &'static str>> =
        HashMap::from([
            (
                "O",
                HashMap::from([
                    ("N", " XX\n XX"),
                    ("E", " XX\n XX"),
                    ("S", " XX\n XX"),
                    ("W", " XX\n XX"),
                ]),
            ),
            (
                "I",
                HashMap::from([
                    ("N", "    \nXXXX\n    \n    "),
                    ("E", "  X \n  X \n  X \n  X "),
                    ("S", "    \n    \nXXXX\n    "),
                    ("W", " X  \n X  \n X  \n X  "),
                ]),
            ),
            (
                "T",
                HashMap::from([
                    ("N", " X \nXXX\n   "),
                    ("E", " X \n XX\n X "),
                    ("S", "   \nXXX\n X "),
                    ("W", " X \nXX \n X "),
                ]),
            ),
            (
                "L",
                HashMap::from([
                    ("N", "  X\nXXX\n   "),
                    ("E", " X \n X \n XX"),
                    ("S", "   \nXXX\nX  "),
                    ("W", "XX \n X \n X "),
                ]),
            ),
            (
                "J",
                HashMap::from([
                    ("N", "X  \nXXX\n   "),
                    ("E", " XX\n X \n X "),
                    ("S", "   \nXXX\n  X"),
                    ("W", " X \n X \nXX "),
                ]),
            ),
            (
                "S",
                HashMap::from([
                    ("N", " XX\nXX \n   "),
                    ("E", " X \n XX\n  X"),
                    ("S", "   \n XX\nXX "),
                    ("W", "X  \nXX \n X "),
                ]),
            ),
            (
                "Z",
                HashMap::from([
                    ("N", "XX \n XX\n   "),
                    ("E", "  X\n XX\n X "),
                    ("S", "   \nXX \n XX"),
                    ("W", " X \nXX \nX  "),
                ]),
            ),
        ]);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rotation {
    N,
    E,
    S,
    W,
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

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

    fn string_to_cells(&self, s: &str) -> Vec<Coordinate> {
        let mut res: Vec<Coordinate> = vec![];
        let lines: Vec<&str> = s.split("\n").collect();

        for (y, line) in lines.into_iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                match c {
                    'X' => res.push(Coordinate {
                        x: x as isize,
                        y: y as isize,
                    }),
                    _ => (),
                }
            }
        }
        res
    }

    fn cells_for(&self, rotation: &Rotation) -> Vec<Coordinate> {
        match self {
            Kind::O => match rotation {
                Rotation::N => self.string_to_cells(SHAPES["O"]["N"]),
                Rotation::E => self.string_to_cells(SHAPES["O"]["E"]),
                Rotation::S => self.string_to_cells(SHAPES["O"]["S"]),
                Rotation::W => self.string_to_cells(SHAPES["O"]["W"]),
            },
            Kind::I => match rotation {
                Rotation::N => self.string_to_cells(SHAPES["I"]["N"]),
                Rotation::E => self.string_to_cells(SHAPES["I"]["E"]),
                Rotation::S => self.string_to_cells(SHAPES["I"]["S"]),
                Rotation::W => self.string_to_cells(SHAPES["I"]["W"]),
            },
            Kind::T => match rotation {
                Rotation::N => self.string_to_cells(SHAPES["T"]["N"]),
                Rotation::E => self.string_to_cells(SHAPES["T"]["E"]),
                Rotation::S => self.string_to_cells(SHAPES["T"]["S"]),
                Rotation::W => self.string_to_cells(SHAPES["T"]["W"]),
            },
            Kind::L => match rotation {
                Rotation::N => self.string_to_cells(SHAPES["L"]["N"]),
                Rotation::E => self.string_to_cells(SHAPES["L"]["E"]),
                Rotation::S => self.string_to_cells(SHAPES["L"]["S"]),
                Rotation::W => self.string_to_cells(SHAPES["L"]["W"]),
            },
            Kind::J => match rotation {
                Rotation::N => self.string_to_cells(SHAPES["J"]["N"]),
                Rotation::E => self.string_to_cells(SHAPES["J"]["E"]),
                Rotation::S => self.string_to_cells(SHAPES["J"]["S"]),
                Rotation::W => self.string_to_cells(SHAPES["J"]["W"]),
            },
            Kind::S => match rotation {
                Rotation::N => self.string_to_cells(SHAPES["S"]["N"]),
                Rotation::E => self.string_to_cells(SHAPES["S"]["E"]),
                Rotation::S => self.string_to_cells(SHAPES["S"]["S"]),
                Rotation::W => self.string_to_cells(SHAPES["S"]["W"]),
            },
            Kind::Z => match rotation {
                Rotation::N => self.string_to_cells(SHAPES["Z"]["N"]),
                Rotation::E => self.string_to_cells(SHAPES["Z"]["E"]),
                Rotation::S => self.string_to_cells(SHAPES["Z"]["S"]),
                Rotation::W => self.string_to_cells(SHAPES["Z"]["W"]),
            },
        }
    }
}

pub struct Piece {
    pub kind: Kind,
    pub position: Coordinate,
    pub rotation: Rotation,
}

impl Piece {
    pub fn get_cells(&self) -> Vec<Coordinate> {
        let shape = self.kind.cells_for(&self.rotation);
        shape
            .iter()
            .map(|mino| Coordinate::new(mino.x + self.position.x, mino.y + self.position.y))
            .collect()
    }

    pub fn cw(&mut self) {
        match self.rotation {
            Rotation::N => self.rotation = Rotation::E,
            Rotation::E => self.rotation = Rotation::S,
            Rotation::S => self.rotation = Rotation::W,
            Rotation::W => self.rotation = Rotation::N,
        }
    }

    pub fn ccw(&mut self) {
        match self.rotation {
            Rotation::N => self.rotation = Rotation::W,
            Rotation::W => self.rotation = Rotation::S,
            Rotation::S => self.rotation = Rotation::E,
            Rotation::E => self.rotation = Rotation::N,
        }
    }

    pub fn lower(&mut self) {
        self.position = Coordinate {
            x: self.position.x,
            y: self.position.y + 1,
        }
    }
}
