use std::str::FromStr;

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Right,
    Direction::Left,
];

#[derive(Debug)]
pub enum GpnError {
    Soft(String),
    Unknown(String),
    ParseError(String),
}

#[derive(Debug)]
pub enum Proto {
    Join {
        name: String,
        password: String,
    },
    Goal {
        pos: Position,
    },
    Pos {
        pos: Position,
        space: Vec<Direction>,
    },
    Move {
        direction: Direction,
    },
    Chat {
        message: String,
    },
    Motd {
        message: String,
    },
    Win {
        wins: i32,
        loses: i32,
    },
    Lose {
        wins: i32,
        loses: i32,
    },
    Game {
        width: i32,
        height: i32,
        goal: Position,
    },
}

impl FromStr for Proto {
    type Err = GpnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args = s.split('|').collect::<Vec<_>>();
        if args.is_empty() {
            return Err(GpnError::Soft(String::from("No args found")));
        }

        return match (args[0], args.len()) {
            ("chat", 2) => Ok(Proto::Chat {
                message: args[1].to_string(),
            }),
            ("motd", 2) => Ok(Proto::Motd {
                message: args[1].to_string(),
            }),
            ("goal", 3) => Ok(Proto::Goal {
                pos: Position::parse(&args[1..])
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
            }),
            ("pos", 7) => Ok(Proto::Pos {
                pos: Position::parse(&[args[1], args[2]])
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
                space: Direction::parse([args[3], args[4], args[5], args[6]]),
            }),
            ("game", 5) => Ok(Proto::Game {
                width: args[1]
                    .parse()
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
                height: args[2]
                    .parse()
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
                goal: Position::parse(&args[3..])
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
            }),
            ("win", 3) => Ok(Proto::Win {
                wins: args[1]
                    .parse()
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
                loses: args[2]
                    .parse()
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
            }),
            ("lose", 3) => Ok(Proto::Lose {
                wins: args[1]
                    .parse()
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
                loses: args[2]
                    .parse()
                    .map_err(|e| GpnError::ParseError(format!("{e}")))?,
            }),
            ("error", _) => Err(GpnError::Soft(format!("Received Error : {}", s))),
            _ => Err(GpnError::Unknown(format!("Unknown Proto message : {}", s))),
        };
    }
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Position {
    pub fn parse(coords: &[&str]) -> Result<Self, std::num::ParseIntError> {
        Ok(Position {
            x: coords[0].parse()?,
            y: coords[1].parse()?,
        })
    }

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn add(&self, d: &Direction) -> Self {
        let x = self.x;
        let y = self.y;
        match d {
            Direction::Up => Self { x, y: y - 1 },
            Direction::Right => Self { x: x + 1, y },
            Direction::Down => Self { x, y: y + 1 },
            Direction::Left => Self { x: x - 1, y },
        }
    }

    pub fn distance(&self, b: &Self) -> f32 {
        (*self - *b).norm()
    }

    pub fn norm(&self) -> f32 {
        ((self.x.pow(2)) as f32 + (self.y.pow(2)) as f32).sqrt()
    }

    pub fn surroundings(&self) -> Vec<Position> {
        DIRECTIONS.iter().map(|d| self.add(d)).collect::<Vec<_>>()
    }
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Position {
        Self { x, y }
    }
}

impl std::ops::Add for Position {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Position {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Direction {
    Left = 1 << 3,
    Up = 1 << 2,
    Right = 1 << 1,
    Down = 1,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Up => "up",
                Self::Right => "right",
                Self::Down => "down",
                Self::Left => "left",
            }
        )
    }
}

impl Direction {
    pub fn parse(args: [&str; 4]) -> Vec<Self> {
        let dic = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        let mut out = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if arg == &"0" {
                out.push(dic[i]);
            }
        }

        out
    }

    pub fn is_dead_end(bits: u8) -> bool {
        bits == Direction::Up as u8 || bits == Direction::Right as u8 || bits == Direction::Left as u8 || bits == Direction::Down as u8
    }

    pub fn inverse(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn into_bits(slice: &[Direction]) -> u8 {
        slice.iter().fold(0u8, |a, c| a | *c as u8)
    }

    pub const EMTPY: u8 = 0;
    pub const FLOODED: u8 = 0b10000;
    pub const START: u8 = 0b100000;
    pub const GOAL: u8 = 0b1000000;

    pub fn into_char(bits: u8) -> char {
        match bits {
            3 => '╔',
            5 => '║',
            6 => '╚',
            7 => '╠',
            9 => '╗',
            10 => '═',
            11 => '╦',
            12 => '╝',
            13 => '╣',
            14 => '╩',
            15 => '╬',
            x if x == Self::FLOODED => '░',
            x if x == Self::START => 'S',
            x if x == Self::GOAL => 'G',
            1 | 2 | 4 | 8 => 'x',
            _ => ' ',
        }
    }
}

type STreeInner = Vec<(Position, Vec<Direction>)>;
pub struct STree(STreeInner);

impl std::ops::Deref for STree {
    type Target = STreeInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for STree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl STree {
    pub fn new() -> Self {
        Self(STreeInner::new())
    }

    pub fn get_keys<'a>(&'a self) -> impl Iterator<Item = &'a Position> + '_ {
        self.iter().map(|(k, _)| k)
    }

    pub fn contains_key(&self, pos: &Position) -> bool {
        self.get_keys().any(|k| k == pos)
    }

    pub fn get_mut(&mut self, pos: &Position) -> Option<&mut Vec<Direction>> {
        self.iter_mut().find(|(k, _)| k == pos).map(|(_, v)| v)
    }

    pub fn get(&self, pos: &Position) -> Option<&Vec<Direction>> {
        self.iter().find(|(k, _)| k == pos).map(|(_, v)| v)
    }
}

pub struct State {
    pub map: Vec<Vec<u8>>,
    pub start: Position,
    pub goal: Position,
    pub size: (i32, i32),
    pub wins: u32,
    pub loses: u32,
}

impl State {
    const MAX_MAP_SIZE: usize = 64;

    pub fn new() -> Self {
        Self {
            map: vec![vec![0; 64]; 64],
            start: Position::new(32, 32),
            goal: Position::new(0, 0),
            size: (0, 0),
            wins: 0,
            loses: 0
        }
    }

    pub fn reset(&mut self, width: i32, height: i32, goal: Position) {
        self.map = vec![vec![0; width as usize + 1]; height as usize + 1];
        self.start = Position::new(0, 0);
        self.goal = goal;
        self.size = (width, height);
    }

    pub fn in_bounds(&self, pos: &Position) -> bool {
        let (x, y) = self.size;
        pos.x <= x && pos.y <= y && pos.x >= 0 && pos.y >= 0
    }
}
