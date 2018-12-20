#![allow(dead_code)]

use std::{
    collections::hash_map::{Entry, HashMap},
    io::{self, Read},
};

fn opposite_dir(dir: char) -> char {
    match dir {
        'N' => 'S',
        'S' => 'N',
        'E' => 'W',
        'W' => 'E',
        _ => panic!("bad character: {}", dir),
    }
}

#[derive(Debug, Default, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[must_use]
struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }

    fn north(self) -> Self {
        Point {
            x: self.x,
            y: self.y + 1,
        }
    }

    fn east(self) -> Self {
        Point {
            x: self.x + 1,
            y: self.y,
        }
    }

    fn west(self) -> Self {
        Point {
            x: self.x - 1,
            y: self.y,
        }
    }

    fn south(self) -> Self {
        Point {
            x: self.x,
            y: self.y - 1,
        }
    }

    fn step(self, dir: char) -> Self {
        match dir {
            'N' => self.north(),
            'E' => self.east(),
            'W' => self.west(),
            'S' => self.south(),
            _ => panic!("bad character: {}", dir),
        }
    }
}

#[derive(Debug, Default, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct Walker {
    pos: Point,
    walked: usize,
}

impl Walker {
    fn walk(&mut self, dir: char) {
        self.walked += 1;
        self.pos = self.pos.step(dir);
    }
}

#[derive(Debug)]
struct Room {
    came_from: char,
    distance: usize,
}

#[derive(Debug)]
struct Snapshot {
    origin: Vec<Walker>,
    branches: Vec<Vec<Walker>>,
}

impl Snapshot {
    fn new(origin: Vec<Walker>) -> Self {
        Snapshot {
            origin,
            branches: Vec::new(),
        }
    }

    fn push_branch(&mut self, walkers: Vec<Walker>) {
        self.branches.push(walkers);
    }

    fn push_branch_and_rewind(&mut self, walkers: &mut Vec<Walker>) {
        let walkers = std::mem::replace(walkers, self.origin.clone());
        self.branches.push(walkers);
    }

    fn extract_walkers(self) -> Vec<Walker> {
        let mut result = Vec::with_capacity(self.branches.iter().map(Vec::len).sum::<usize>());
        self.branches
            .into_iter()
            .for_each(|mut branch| result.append(&mut branch));
        // Sort so that walkers at the same position appear subsequent in the
        // list. And so that the walker with the lowest distance appears first.
        result.sort();
        // Remove all walkers that appear at the same position.
        result.dedup_by_key(|walker| walker.pos);
        result
    }
}

fn handle_direction(dir: char, walkers: &mut [Walker], rooms: &mut HashMap<Point, Room>) {
    for walker in walkers {
        walker.walk(dir);
        let room = Room {
            came_from: opposite_dir(dir),
            distance: walker.walked,
        };
        match rooms.entry(walker.pos) {
            Entry::Vacant(entry) => {
                entry.insert(room);
            },
            Entry::Occupied(mut entry) => {
                if walker.walked < entry.get().distance {
                    *entry.get_mut() = room;
                }
            },
        }
    }
}

fn walk_path(path: &str) -> HashMap<Point, Room> {
    let mut rooms = HashMap::<Point, Room>::new();
    let mut walkers = vec![Walker::default()];
    let mut snapshots = Vec::<Snapshot>::new();
    for c in path.chars() {
        match c {
            '(' => {
                snapshots.push(Snapshot::new(walkers.clone()));
            },
            '|' => {
                snapshots
                    .last_mut()
                    .unwrap()
                    .push_branch_and_rewind(&mut walkers);
            },
            ')' => {
                let mut snapshot = snapshots.pop().unwrap();
                snapshot.push_branch(std::mem::replace(&mut walkers, Vec::new()));
                walkers = snapshot.extract_walkers();
            },
            c => handle_direction(c, &mut walkers, &mut rooms),
        }
    }
    assert!(snapshots.is_empty());
    rooms
}

fn distance_to_furthest_room(rooms: &HashMap<Point, Room>) -> usize {
    let mut pos = rooms
        .iter()
        .max_by_key(|(_, Room { distance, .. })| distance)
        .map(|(&pos, _)| pos)
        .unwrap();
    let mut steps = 0;
    while pos != Point::default() {
        pos = pos.step(rooms[&pos].came_from);
        steps += 1;
    }
    steps
}

fn main() {
    let path: String = {
        let mut buf = String::with_capacity(14100);
        io::stdin().read_to_string(&mut buf).unwrap();
        buf
    };
    let rooms = walk_path(path.trim_start_matches('^').trim_end_matches("$\n"));
    println!("part 1: {}", distance_to_furthest_room(&rooms));
}
