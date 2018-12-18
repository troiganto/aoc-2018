use boolinator::Boolinator;
use std::{
    fmt,
    io::{self, Read},
    str::FromStr,
};

#[derive(Debug)]
pub struct BadTile(char);

#[derive(Debug)]
pub enum BadMap {
    BadWidth { expected: usize, actual: usize },
    ZeroWidth,
    BadTile(BadTile),
}

impl From<BadTile> for BadMap {
    fn from(err: BadTile) -> BadMap {
        BadMap::BadTile(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Tile {
    Open = b'.',
    Tree = b'|',
    Yard = b'#',
}

impl Tile {
    fn from_char(c: char) -> Result<Self, BadTile> {
        let tile = match c {
            '.' => Tile::Open,
            '|' => Tile::Tree,
            '#' => Tile::Yard,
            _ => Err(BadTile(c))?,
        };
        Ok(tile)
    }
}

impl From<Tile> for char {
    fn from(tile: Tile) -> char {
        match tile {
            Tile::Open => '.',
            Tile::Tree => '|',
            Tile::Yard => '#',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Map {
    tiles: Vec<Tile>,
    backup: Vec<Tile>,
    width: usize,
    height: usize,
}

impl Map {
    fn step_one_minute(&mut self) {
        let mut backup = std::mem::replace(&mut self.backup, Vec::new());
        let iter = self
            .tiles
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, tile)| -> Tile {
                let neighbors = self.neighbors(i);
                match tile {
                    Tile::Open => {
                        if neighbors.filter(|&&tile| tile == Tile::Tree).count() >= 3 {
                            Tile::Tree
                        } else {
                            Tile::Open
                        }
                    },
                    Tile::Tree => {
                        if neighbors.filter(|&&tile| tile == Tile::Yard).count() >= 3 {
                            Tile::Yard
                        } else {
                            Tile::Tree
                        }
                    },
                    Tile::Yard => {
                        let (seen_yard, seen_tree) = neighbors.fold(
                            (false, false),
                            |(mut seen_yard, mut seen_tree), &tile| {
                                match tile {
                                    Tile::Tree => seen_tree = true,
                                    Tile::Yard => seen_yard = true,
                                    _ => {},
                                }
                                (seen_yard, seen_tree)
                            },
                        );
                        if seen_tree && seen_yard {
                            Tile::Yard
                        } else {
                            Tile::Open
                        }
                    },
                }
            });
        backup.splice(.., iter);
        std::mem::swap(&mut backup, &mut self.tiles);
        self.backup = backup;
    }

    fn num_yards(&self) -> usize {
        self.tiles
            .iter()
            .filter(|&&tile| tile == Tile::Yard)
            .count()
    }

    fn num_trees(&self) -> usize {
        self.tiles
            .iter()
            .filter(|&&tile| tile == Tile::Tree)
            .count()
    }

    fn total_resource_value(&self) -> usize {
        self.num_trees() * self.num_yards()
    }

    fn neighbors(&self, i: usize) -> Neighbors<'_> {
        Neighbors {
            map: self,
            pos: i,
            i: 0,
        }
    }
}

impl FromStr for Map {
    type Err = BadMap;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut width = 0;
        let mut height = 0;
        let mut tiles = Vec::with_capacity(s.len());
        let backup = Vec::with_capacity(tiles.capacity());
        for (i, c) in s.char_indices() {
            match c {
                '\n' => {
                    if width == 0 {
                        (i > 0).ok_or(BadMap::ZeroWidth)?;
                        width = i;
                    } else {
                        ((i + 1) % (width + 1) == 0).ok_or(BadMap::BadWidth {
                            expected: width,
                            actual: i,
                        })?;
                    }
                    height += 1;
                },
                c => tiles.push(Tile::from_char(c)?),
            }
        }
        Ok(Map {
            tiles,
            backup,
            width,
            height,
        })
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        for line in self.tiles.chunks(self.width) {
            for &tile in line {
                f.write_char(tile.into())?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Neighbors<'a> {
    map: &'a Map,
    pos: usize,
    i: usize,
}

impl<'a> Iterator for Neighbors<'a> {
    type Item = &'a Tile;

    fn next(&mut self) -> Option<&'a Tile> {
        loop {
            let &Map { width, height, .. } = self.map;
            let &mut Neighbors { pos, i, .. } = self;
            let x = pos % width;
            let y = pos / width;
            let item = match i {
                0 => (y > 0 && x > 0).as_some_from(|| pos - width - 1),
                1 => (y > 0).as_some_from(|| pos - width),
                2 => (y > 0 && x < width - 1).as_some_from(|| pos - width + 1),
                3 => (x > 0).as_some_from(|| pos - 1),
                4 => (x < width - 1).as_some_from(|| pos + 1),
                5 => (y < height - 1 && x > 0).as_some_from(|| pos + width - 1),
                6 => (y < height - 1).as_some_from(|| pos + width),
                7 => (y < height - 1 && x < width - 1).as_some_from(|| pos + width + 1),
                _ => break None,
            };
            self.i += 1;
            if let Some(i) = item {
                break self.map.tiles.get(i);
            }
        }
    }
}

pub fn simulator(mut map: Map, minutes: usize) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    writeln!(&mut output, "Initial state:\n{}", map);
    if minutes > 0 {
        map.step_one_minute();
        writeln!(&mut output, "After 1 minute:\n{}", map);
    }
    for i in 2..=minutes {
        map.step_one_minute();
        writeln!(&mut output, "After {} minutes:\n{}", i, map);
    }
    output
}

pub fn cycle_len_at_end<T: PartialEq>(items: &[T]) -> usize {
    for len in 1..=items.len() / 2 {
        let mut chunks = items.rchunks(len);
        match (chunks.next(), chunks.next()) {
            (Some(l), Some(r)) if l == r => return len,
            _ => {},
        }
    }
    0
}

fn main() {
    let mut map: Map = {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        buf.parse().unwrap()
    };
    let mut resources = Vec::with_capacity(1000);
    for i in 0..1000 {
        if i == 10 {
            println!("after 10 minutes: {}", map.total_resource_value());
        }
        resources.push((map.num_trees(), map.num_yards()));
        map.step_one_minute();
    }
    let cycle_len = {
        let cycle_len = cycle_len_at_end(&resources);
        if cycle_len == 0 {
            panic!("no cycle found");
        }
        let mut clone = map.clone();
        for _ in 0..cycle_len {
            clone.step_one_minute();
        }
        if clone != map {
            panic!("cycle not reliable");
        }
        cycle_len
    };
    println!("cycle length: {}", cycle_len);
    let cycle = &resources[resources.len() - cycle_len..];
    assert_eq!(cycle[0], (map.num_trees(), map.num_yards()));
    let final_values = cycle[(1_000_000_000 - resources.len()) % cycle.len()];
    println!(
        "after 1_000_000_000 minutes: {}",
        final_values.0 * final_values.1,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map() {
        use self::Tile::*;
        let map: Map = ".|#\n..#\n#||\n".parse().unwrap();
        let neighbors: Vec<Vec<Tile>> = (0..9)
            .map(|i| map.neighbors(i).cloned().collect())
            .collect();
        assert_eq!(
            neighbors,
            &[
                &[Tree, Open, Open][..],
                &[Open, Yard, Open, Open, Yard],
                &[Tree, Open, Yard],
                &[Open, Tree, Open, Yard, Tree],
                &[Open, Tree, Yard, Open, Yard, Yard, Tree, Tree],
                &[Tree, Yard, Open, Tree, Tree],
                &[Open, Open, Tree],
                &[Open, Open, Yard, Yard, Tree],
                &[Open, Yard, Tree],
            ][..]
        );
    }

    #[test]
    fn test_example() {
        let map: Map = ".#.#...|#.
.....#|##|
.|..|...#.
..|#.....#
#.#|||#|#|
...#.||...
.|....|...
||...#|.#|
|.||||..|.
...#.|..|.
"
        .parse()
        .unwrap();
        assert_eq!(
            simulator(map, 10).lines().collect::<Vec<_>>(),
            "Initial state:
.#.#...|#.
.....#|##|
.|..|...#.
..|#.....#
#.#|||#|#|
...#.||...
.|....|...
||...#|.#|
|.||||..|.
...#.|..|.

After 1 minute:
.......##.
......|###
.|..|...#.
..|#||...#
..##||.|#|
...#||||..
||...|||..
|||||.||.|
||||||||||
....||..|.

After 2 minutes:
.......#..
......|#..
.|.|||....
..##|||..#
..###|||#|
...#|||||.
|||||||||.
||||||||||
||||||||||
.|||||||||

After 3 minutes:
.......#..
....|||#..
.|.||||...
..###|||.#
...##|||#|
.||##|||||
||||||||||
||||||||||
||||||||||
||||||||||

After 4 minutes:
.....|.#..
...||||#..
.|.#||||..
..###||||#
...###||#|
|||##|||||
||||||||||
||||||||||
||||||||||
||||||||||

After 5 minutes:
....|||#..
...||||#..
.|.##||||.
..####|||#
.|.###||#|
|||###||||
||||||||||
||||||||||
||||||||||
||||||||||

After 6 minutes:
...||||#..
...||||#..
.|.###|||.
..#.##|||#
|||#.##|#|
|||###||||
||||#|||||
||||||||||
||||||||||
||||||||||

After 7 minutes:
...||||#..
..||#|##..
.|.####||.
||#..##||#
||##.##|#|
|||####|||
|||###||||
||||||||||
||||||||||
||||||||||

After 8 minutes:
..||||##..
..|#####..
|||#####|.
||#...##|#
||##..###|
||##.###||
|||####|||
||||#|||||
||||||||||
||||||||||

After 9 minutes:
..||###...
.||#####..
||##...##.
||#....###
|##....##|
||##..###|
||######||
|||###||||
||||||||||
||||||||||

After 10 minutes:
.||##.....
||###.....
||##......
|##.....##
|##.....##
|##....##|
||##.####|
||#####|||
||||#|||||
||||||||||

"
            .lines()
            .collect::<Vec<_>>(),
        );
    }
}
