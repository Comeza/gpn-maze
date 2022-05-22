#![allow(dead_code)]

const DRAW: bool = true;

use proto::*;
use tokio::io::AsyncRead;
use tokio::task::JoinHandle;
use std::collections::HashSet;
use std::future::Future;
use std::io::{BufRead, BufReader, Write, Error};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::Duration;
use std::pin::Pin;

mod proto;

type StateType = Arc<Mutex<State>>;
struct Login<'a>(&'a str, &'a str);

enum PrioMode {
    Distance,
    Random,
    Deterministic([Direction; 4])
}

struct BotConfig {
    username: String,
    password: String,
    index: u32,
    prio: PrioMode,
}

impl BotConfig {
    fn new(username: &str, password: &str, index: u32, prio: PrioMode) -> Self {
        Self {username: username.to_string(), password: password.to_string(), index, prio}
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    let ip = args.get(1).expect("No Ip given").to_owned();

    let state = Arc::new(Mutex::new(State::new()));
    
    use Direction::*;
    let bots = vec![
        BotConfig::new("Staubichsauger", "pw", 0, PrioMode::Distance),
        BotConfig::new("Kocherwasser", "pw", 1, PrioMode::Random),
        BotConfig::new("Eisenbuegel", "pw", 2, PrioMode::Deterministic([Up, Right, Left, Down])),
        BotConfig::new("Schrankkuel", "pw", 2, PrioMode::Deterministic([Left, Right, Up, Down])),
        BotConfig::new("Messerkuechen", "pw", 2, PrioMode::Deterministic([Right, Up, Left, Down])),
    ];
    
    for config in bots {
        let state = state.clone();
        let ip = ip.clone();
        tokio::spawn(async move { bot_init(config, state, ip).unwrap(); });
    }
    loop {
        if DRAW {
            draw_sate(&state.lock().expect("Could not lock state"));
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

fn bot_init(config: BotConfig, state: StateType, ip: String) -> std::io::Result<()> {
    let mut stree = STree::new();
    let mut stream = BufReader::new(TcpStream::connect(ip)?);
    let mut out = String::new();
    

    stream.read_line(&mut out)?;

    println!("connected: {}", config.username);
    writeln!(stream.get_mut(), "join|{}|{}", config.username, config.password)?;

    let mut path = Vec::<Direction>::new();

    loop {
        let mut line = String::new();
        if stream.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.replace('\n', "");
        if !DRAW {
            print!(" ({}) -> {line}", config.username);
        }

        match Proto::from_str(&line) {
            Ok(p) => { 
                let mut state = state.lock().expect("Could not get mut state");
                match p {
                Proto::Pos { pos, space } if !space.is_empty() => {
                    // println!(" :: {pos:?} -- {:?}", path.len());
                    state.map[pos.x as usize][pos.y as usize] = if stree.is_empty() {
                        Direction::START
                    } else {
                        Direction::into_bits(&space)
                    };

                    let mut space = Vec::from(space);
                    match config.prio {
                        PrioMode::Random => space.shuffle(&mut thread_rng()),
                        PrioMode::Distance => space
                            .sort_unstable_by_key(|d| -(pos.add(d).distance(&state.goal) * 100f32) as i32),
                        PrioMode::Deterministic(seq) => { space
                            .sort_unstable_by_key(|sd| -(seq.iter().position(|d| sd == d).unwrap() as i32));
                        },
                        _=> {}
                    }

                    if !stree.contains_key(&pos) {
                        match path.last() {
                            None => {
                                state.start = pos;
                                stree.push((pos, space))
                            }
                            Some(last) => stree.push((
                                pos,
                                space
                                    .into_iter()
                                    .filter(|dir| *dir != last.inverse())
                                    .collect(),
                            )),
                        }
                    }

                    let possible_dirs = stree.get_mut(&pos).unwrap();
                    if possible_dirs.is_empty() && path.is_empty() { 
                        write!(stream.get_mut(), "chat|im stuck :/");
                        return Err(Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe :(")); 
                    }
                    let dir = if (possible_dirs.is_empty() || state.map[pos.add(&possible_dirs.last().unwrap()).x as usize][pos.add(&possible_dirs.last().unwrap()).y as usize] == Direction::FLOODED) && !path.is_empty() {
                        path.pop().unwrap().inverse()
                    } else {
                        let dir = possible_dirs.pop().unwrap();

                        let flood = flood(&mut state, &stree, &pos.add(&dir));
                        if !flood.is_empty() && !flood.contains(&state.goal){
                            for entry in flood {
                                state.map[entry.x as usize][entry.y as usize] = Direction::FLOODED;
                            }
                            path.pop().unwrap().inverse()
                        } else {
                            path.push(dir);
                            dir
                        }
                    };

                    writeln!(stream.get_mut(), "move|{dir}")?;
                }
                Proto::Game {
                    width,
                    height,
                    goal,
                } => {
                    state.reset(width, height, goal);
                    state.map[goal.x as usize][goal.y as usize] = Direction::GOAL;
                    stree = STree::new();
                }
                Proto::Win { wins, loses } | Proto::Lose { wins, loses } => {
                    if wins as f32 / loses as f32 > state.wins as f32 / state.loses as f32 {
                        state.wins = wins as u32;
                        state.loses = loses as u32;
                    }
                }
                _ => {}
            }},
            Err(e) => {
                eprintln!(" !! <{line}> {e:?}");
            }
        }
    }
    Ok(())
}

fn draw_sate(state: &State) {
    let State {
        goal,
        start,
        size: (width, height),
        ..
    } = state;
    
    println!();
    print!("{}[2J", 27 as char);

    // state.map[goal.x as usize][goal.y as usize] = Direction::GOAL;
    let mut flooded_count = 0;
    for y in -1..=*width {
        for x in -1..=*height {
            if x == -1 || y == -1 || x == *width || y == *height {
                print!("█");
                continue;
            }
            let tile = state.map[x as usize][y as usize];
            if tile == Direction::FLOODED {
                flooded_count += 1;
            }
            print!("{}", Direction::into_char(tile));
        }
        println!();
    }
    println!("Map Info: ({}x{})", width, height);
    println!("Flooded: {}", flooded_count);
    println!("Wins {} Loses {} ({:.4})", state.wins, state.loses, state.wins as f32 / state.loses as f32)
}

fn flood(state: &mut State, stree: &STree, curr: &Position) -> HashSet<Position> {
    let mut search = vec![*curr];
    let mut flood = HashSet::new();

    while !search.is_empty() {
        let elem = search.pop().unwrap();

        if !state.in_bounds(&elem)
            || (state.map[elem.x as usize][elem.y as usize] != Direction::EMTPY && state.map[elem.x as usize][elem.y as usize] != Direction::GOAL)
            || flood.contains(&elem)
        {
            continue;
        }

        search.extend_from_slice(&elem.surroundings());
        flood.insert(elem);

        if elem == state.goal {
            break;
        }
        /*
        if flood.len() >= 32 {
            flood.insert(state.goal);
            break;
        }
        */
    }

    flood
}
