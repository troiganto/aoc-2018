use boolinator::Boolinator;
use std::{
    collections::HashMap,
    io::{self, Read},
};

#[derive(Debug, Default, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    #[must_use]
    pub fn up(self) -> Option<Self> {
        let Point { x, y } = self;
        y.checked_sub(1).map(|y| Point { x, y })
    }

    #[must_use]
    pub fn down(self) -> Option<Self> {
        let Point { x, y } = self;
        y.checked_add(1).map(|y| Point { x, y })
    }

    #[must_use]
    pub fn left(self) -> Option<Self> {
        let Point { x, y } = self;
        x.checked_sub(1).map(|x| Point { x, y })
    }

    #[must_use]
    pub fn right(self) -> Option<Self> {
        let Point { x, y } = self;
        x.checked_add(1).map(|x| Point { x, y })
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Tool {
    Torch,
    Climbing,
    Neither,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Torch
    }
}

impl Tool {
    #[must_use]
    pub fn switch(self, tile: &Tile) -> Self {
        match (tile, self) {
            (Tile::Rocky, Tool::Climbing) => Tool::Torch,
            (Tile::Rocky, Tool::Torch) => Tool::Climbing,
            (Tile::Wet, Tool::Climbing) => Tool::Neither,
            (Tile::Wet, Tool::Neither) => Tool::Climbing,
            (Tile::Narrow, Tool::Torch) => Tool::Neither,
            (Tile::Narrow, Tool::Neither) => Tool::Torch,
            _ => panic!("bad tool for region: {:?} in {:?}", self, tile),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Tile {
    Rocky,
    Wet,
    Narrow,
}

pub fn is_compatible(tool: &Tool, tile: &Tile) -> bool {
    match (tool, tile) {
        (Tool::Neither, Tile::Rocky)
        | (Tool::Torch, Tile::Wet)
        | (Tool::Climbing, Tile::Narrow) => false,
        _ => true,
    }
}

impl Tile {
    pub fn risk(&self) -> u8 {
        match self {
            Tile::Rocky => 0,
            Tile::Wet => 1,
            Tile::Narrow => 2,
        }
    }
}

pub struct Scanlines {
    depth: usize,
    size: Point,
    prev_erosion: Vec<usize>,
    y: usize,
}

impl Iterator for Scanlines {
    type Item = Vec<Tile>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.size.y {
            return None;
        }
        let erosion = if self.prev_erosion.is_empty() {
            (0..self.size.x)
                .map(|x| x * 16807)
                .map(|geo_index| (geo_index + self.depth) % 20183)
                .collect()
        } else {
            self.prev_erosion
                .iter()
                .cloned()
                .scan(None, |left_erosion, above_erosion| {
                    let erosion = if let Some(left_erosion) = *left_erosion {
                        let geo_index = left_erosion * above_erosion;
                        (geo_index + self.depth) % 20183
                    } else {
                        let geo_index = self.y * 48271;
                        (geo_index + self.depth) % 20183
                    };
                    *left_erosion = Some(erosion);
                    Some(erosion)
                })
                .collect()
        };
        self.y += 1;
        self.prev_erosion = erosion;
        Some(
            self.prev_erosion
                .iter()
                .map(|erosion| match erosion % 3 {
                    0 => Tile::Rocky,
                    1 => Tile::Wet,
                    2 => Tile::Narrow,
                    _ => unreachable!(),
                })
                .collect(),
        )
    }
}

fn iter_scanlines(depth: usize, size: Point) -> Scanlines {
    Scanlines {
        size,
        depth,
        y: 0,
        prev_erosion: Vec::new(),
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State {
    pos: Point,
    tool: Tool,
}

impl State {
    pub fn distance(&self, other: &Self) -> usize {
        let dx = if self.pos.x > other.pos.x {
            self.pos.x - other.pos.x
        } else {
            other.pos.x - self.pos.x
        };
        let dy = if self.pos.y > other.pos.y {
            self.pos.y - other.pos.y
        } else {
            other.pos.y - self.pos.y
        };
        let dt = if self.tool == other.tool { 0 } else { 7 };
        dx + dy + dt
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    tiles: Vec<Tile>,
    size: Point,
    target: Point,
}

impl Map {
    pub fn new(depth: usize, size: Point, target: Point) -> Self {
        let tiles = iter_scanlines(depth, size).flatten().collect();
        let mut map = Map {
            tiles,
            size,
            target,
        };
        map[target] = Tile::Rocky;
        map
    }

    pub fn neighbors(&self, center: State) -> Neighbors {
        Neighbors {
            map: self,
            center,
            i: 0,
        }
    }

    pub fn find_route(&self) -> Option<usize> {
        let target = State {
            pos: self.target,
            tool: Tool::Torch,
        };
        let mut visited = HashMap::<State, usize>::new();
        let mut open = HashMap::<State, usize>::new();
        open.insert(State::default(), 0);
        visited.insert(State::default(), 0);
        while let Some((&current, &time_taken)) = open
            .iter()
            .min_by_key(|&(state, &time_taken)| time_taken + state.distance(&target))
        {
            open.remove(&current);
            if current == target {
                return Some(time_taken);
            }
            for neighbor in self.neighbors(current) {
                let new_time = time_taken + current.distance(&neighbor);
                visited
                    .entry(neighbor)
                    .and_modify(|old_record| {
                        if new_time < *old_record {
                            open.insert(neighbor, new_time);
                            *old_record = new_time;
                        }
                    })
                    .or_insert_with(|| {
                        open.insert(neighbor, new_time);
                        new_time
                    });
            }
        }
        None
    }
}

impl std::ops::Index<Point> for Map {
    type Output = Tile;
    fn index(&self, idx: Point) -> &Self::Output {
        assert!(
            idx.x < self.size.x && idx.y < self.size.y,
            "{:?} out of bounds for map of size {:?}",
            idx,
            self.size,
        );
        &self.tiles[idx.y * self.size.x + idx.x]
    }
}

impl std::ops::IndexMut<Point> for Map {
    fn index_mut(&mut self, idx: Point) -> &mut Self::Output {
        assert!(
            idx.x < self.size.x && idx.y < self.size.y,
            "{:?} out of bounds for map of size {:?}",
            idx,
            self.size,
        );
        &mut self.tiles[idx.y * self.size.x + idx.x]
    }
}

pub struct Neighbors<'a> {
    map: &'a Map,
    center: State,
    i: usize,
}

impl<'a> Iterator for Neighbors<'a> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let State { pos, tool } = self.center;
        loop {
            let item = match self.i {
                0 => pos.up().and_then(|pos| {
                    is_compatible(&tool, &self.map[pos]).as_some(State { pos, tool })
                }),
                1 => pos.down().and_then(|pos| {
                    is_compatible(&tool, &self.map[pos]).as_some(State { pos, tool })
                }),
                2 => pos.left().and_then(|pos| {
                    is_compatible(&tool, &self.map[pos]).as_some(State { pos, tool })
                }),
                3 => pos.right().and_then(|pos| {
                    is_compatible(&tool, &self.map[pos]).as_some(State { pos, tool })
                }),
                4 => Some(State {
                    pos,
                    tool: tool.switch(&self.map[pos]),
                }),
                _ => break None,
            };
            self.i += 1;
            if item.is_some() {
                break item;
            }
        }
    }
}

fn parse_input(s: &str) -> (usize, Point) {
    let mut words = s.split_whitespace();
    assert_eq!(words.next(), Some("depth:"));
    let depth: usize = words.next().unwrap().parse().unwrap();
    assert_eq!(words.next(), Some("target:"));
    let mut words = words.next().unwrap().splitn(2, ',');
    let x: usize = words.next().unwrap().parse().unwrap();
    let y: usize = words.next().unwrap().parse().unwrap();
    assert!(words.next().is_none());
    (depth, Point { x, y })
}

fn main() {
    let (depth, target) = {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        parse_input(&buf)
    };
    let risk = Map::new(
        depth,
        Point {
            x: target.x + 1,
            y: target.y + 1,
        },
        target,
    )
    .tiles
    .iter()
    .map(|t| t.risk() as u32)
    .sum::<u32>();
    println!("risk: {}", risk);
    let time = Map::new(depth, Point { x: 1500, y: 1500 }, target)
        .find_route()
        .unwrap();
    println!("time: {}", time);
}
