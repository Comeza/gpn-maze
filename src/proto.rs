use std::str::FromStr;

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
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Direction {
    Left = 1<<3,
    Up = 1<<2,
    Right = 1<<1,
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

        return out;
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
        slice.iter().fold(0u8, |a,c| a | *c as u8)
    }

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
            1 | 2 | 3 | 4 => 'x',
            _ => ' '
        }
    }
}
