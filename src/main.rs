#![allow(dead_code)]

use proto::*;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str::FromStr;

mod proto;

type STreeInner = Vec<(Position, Vec<Direction>)>;
struct STree(STreeInner);

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
    fn new() -> Self {
        Self(STreeInner::new())
    }

    fn get_keys<'a>(&'a self) -> impl Iterator<Item = &'a Position> {
        self.iter().map(|(k, _)| k)
    }

    fn contains_key(&self, pos: &Position) -> bool {
        self.get_keys().any(|k| k == pos)
    }

    fn get_mut(&mut self, pos: &Position) -> Option<&mut Vec<Direction>> {
        self.iter_mut().find(|(k, _)| k == pos).map(|(_, v)| v)
    }

    fn get(&self, pos: &Position) -> Option<&Vec<Direction>> {
        self.iter().find(|(k, _)| k == pos).map(|(_, v)| v)
    }
}

fn main() -> std::io::Result<()> {
    println!("POS: {:?}", Position::new(0, 0).add(&Direction::Right));
    let mut stream = BufReader::new(TcpStream::connect("94.45.252.221:4000")?);
    let mut out = String::new();
    stream.read_line(&mut out)?;

    println!("CONNECT: {}", out);
    writeln!(stream.get_mut(), "join|Staubichsauger|unendlich")?;

    let mut stree = STree::new();
    let mut path = Vec::<Direction>::new();
    let mut maps = Vec::<[[u8; 32]; 32]>::new();

    let mut current_map = [[0; 32]; 32];
    loop {
        let mut line = String::new();
        if stream.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.replace('\n', "");

        match Proto::from_str(&line) {
            Ok(p) => {
                match p {
                    Proto::Pos { pos, space } if !space.is_empty() => {
//                        println!(" :: {pos:?} -- {} ", path.len());
                        current_map[pos.x as usize][pos.y as usize] = Direction::into_bits(&space);

                        let mut space = Vec::from(space);
                        space.sort_unstable_by_key(|s| *s as u8);

                        if !stree.contains_key(&pos) {
                            match path.last() {
                                None => stree.push((pos, space)),
                                Some(last) => stree.push((
                                    pos,
                                    space
                                        .into_iter()
                                        .filter(|dir| *dir != last.inverse())
                                        .collect(),
                                )),
                            }
                        }

                        let last = &mut stree.get_mut(&pos).unwrap();
                        let dir = if last.is_empty() {
                            path.pop().unwrap().inverse()
                        } else {
                            let dir = last.pop().unwrap();
                            path.push(dir);
                            dir
                        };

                        writeln!(stream.get_mut(), "move|{dir}")?;

                        // TODO: Zurücklaufen
                    }
                    Proto::Goal { .. } => {
                        println!(" :: RESET");

                        for y in 0..32 {
                            for x in 0..32 {
                                // print!("{:0>4} ", current_map[x][y]);
                                print!("{}", Direction::into_char(current_map[x][y]));
                            }
                            println!("");
                        }

                        maps.push(current_map);
                        current_map = [[0u8; 32]; 32];
                        stree.clear();
                        path.clear();
                    },
                    Proto::Win { wins, loses} | Proto::Lose {wins, loses} => {
                        println!(" >> {wins}w {loses}l")
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!(" !! <{line}> {e:?}"),
        }
    }
    Ok(())
}
