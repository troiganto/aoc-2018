use std::{
    collections::hash_map::{Entry, HashMap},
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

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Tile {
    Rocky,
    Wet,
    Narrow,
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
}

impl std::ops::Index<Point> for Map {
    type Output = Tile;
    fn index(&self, idx: Point) -> &Self::Output {
        &self.tiles[idx.y * self.size.x + idx.x]
    }
}

impl std::ops::IndexMut<Point> for Map {
    fn index_mut(&mut self, idx: Point) -> &mut Self::Output {
        &mut self.tiles[idx.y * self.size.x + idx.x]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State {
    pos: Point,
    tool: Tool,
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
}
