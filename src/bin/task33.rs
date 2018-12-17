use boolinator::Boolinator;
use std::{
    cmp::{max, min, Ord},
    fmt,
    io::{self, BufRead},
    num::ParseIntError,
    ops::Range,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum FlowDir {
    Left,
    Right,
    Down,
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Point {
    y: usize,
    x: usize,
}

impl Point {
    #[must_use]
    pub fn stepn(self, n: usize, dir: FlowDir) -> Self {
        match dir {
            FlowDir::Left => Point {
                x: self.x - n,
                y: self.y,
            },
            FlowDir::Right => Point {
                x: self.x + n,
                y: self.y,
            },
            FlowDir::Down => Point {
                x: self.x,
                y: self.y + n,
            },
        }
    }

    #[must_use]
    pub fn step(self, dir: FlowDir) -> Self {
        self.stepn(1, dir)
    }

    #[must_use]
    pub fn above(self) -> Self {
        Point {
            x: self.x,
            y: self.y - 1,
        }
    }

    #[must_use]
    pub fn left(self) -> Self {
        self.step(FlowDir::Left)
    }

    #[must_use]
    pub fn right(self) -> Self {
        self.step(FlowDir::Right)
    }

    #[must_use]
    pub fn below(self) -> Self {
        self.step(FlowDir::Down)
    }
}

#[derive(Debug, Clone)]
pub enum Line {
    Horizontal { y: usize, x: Range<usize> },
    Vertical { x: usize, y: Range<usize> },
}

#[derive(Debug, Clone)]
pub enum BadLine {
    BadCoords,
    MissingPart,
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for BadLine {
    fn from(err: ParseIntError) -> Self {
        BadLine::ParseIntError(err)
    }
}

impl FromStr for Line {
    type Err = BadLine;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ", ");
        let part = parts.next().ok_or(BadLine::MissingPart)?;
        let first_letter = part.chars().nth(0).ok_or(BadLine::MissingPart)?;
        let first: usize = part[2..].parse()?;
        let part = parts.next().ok_or(BadLine::MissingPart)?;
        assert!(parts.next().is_none());
        let second_letter = part.chars().nth(0).ok_or(BadLine::MissingPart)?;
        let mut parts = part[2..].splitn(2, "..");
        let second_from: usize = parts.next().ok_or(BadLine::MissingPart)?.parse()?;
        let second_to: usize = parts.next().ok_or(BadLine::MissingPart)?.parse()?;
        assert!(parts.next().is_none());
        let line = match (first_letter, second_letter) {
            ('x', 'y') => Line::Vertical {
                x: first,
                y: second_from..second_to + 1,
            },
            ('y', 'x') => Line::Horizontal {
                x: second_from..second_to + 1,
                y: first,
            },
            _ => Err(BadLine::BadCoords)?,
        };
        Ok(line)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    horizontal: Range<usize>,
    vertical: Range<usize>,
}

impl BoundingBox {
    pub fn new(line: &Line) -> Self {
        match line {
            &Line::Horizontal { ref x, y } => BoundingBox {
                horizontal: x.clone(),
                vertical: y..y + 1,
            },
            &Line::Vertical { x, ref y } => BoundingBox {
                horizontal: x..x + 1,
                vertical: y.clone(),
            },
        }
    }

    pub fn width(&self) -> usize {
        self.horizontal.end - self.horizontal.start
    }

    pub fn height(&self) -> usize {
        self.vertical.end - self.vertical.start
    }

    pub fn contains(&self, p: Point) -> bool {
        self.horizontal.start <= p.x
            && p.x < self.horizontal.end
            && self.vertical.start <= p.y
            && p.y < self.vertical.end
    }

    pub fn get_relative(&self, p: Point) -> Option<Point> {
        self.contains(p).as_option()?;
        Some(Point {
            x: p.x - self.horizontal.start,
            y: p.y - self.vertical.start,
        })
    }

    pub fn add_line(&mut self, line: &Line) {
        fn update_min<T: Ord + Copy>(first: &mut T, second: T) {
            *first = min(*first, second);
        }
        fn update_max<T: Ord + Copy>(first: &mut T, second: T) {
            *first = max(*first, second);
        }
        match line {
            Line::Horizontal { x, y } => {
                update_min(&mut self.horizontal.start, x.start);
                update_max(&mut self.horizontal.end, x.end);
                update_min(&mut self.vertical.start, *y);
                update_max(&mut self.vertical.end, y + 1);
            },
            Line::Vertical { x, y } => {
                update_min(&mut self.horizontal.start, *x);
                update_max(&mut self.horizontal.end, x + 1);
                update_min(&mut self.vertical.start, y.start);
                update_max(&mut self.vertical.end, y.end);
            },
        }
    }
}

impl<'a> std::iter::Extend<&'a Line> for BoundingBox {
    fn extend<T: IntoIterator<Item = &'a Line>>(&mut self, iter: T) {
        for line in iter {
            self.add_line(line);
        }
    }
}

impl<'a> std::iter::FromIterator<&'a Line> for Option<BoundingBox> {
    fn from_iter<T: IntoIterator<Item = &'a Line>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let mut bb = BoundingBox::new(iter.next()?);
        bb.extend(iter);
        Some(bb)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Tile {
    Sand,
    Clay,
    StillWater,
    FlowingWater,
}

impl Tile {
    pub fn to_char(&self) -> char {
        match self {
            Tile::Clay => '#',
            Tile::Sand => '.',
            Tile::StillWater => '~',
            Tile::FlowingWater => '|',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchResult {
    /// A connection to either the bottom edge or flowing water has been made.
    End(usize),
    /// The flow has been stopped by an obstacle.
    Stopped(usize),
    /// The flow must spill over an edge.
    Overflow(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WaterSummary {
    num_still: usize,
    num_flowing: usize,
}

#[derive(Debug, Clone)]
pub struct Map {
    bounds: BoundingBox,
    source: Point,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(lines: &[Line]) -> Self {
        let mut bounds = lines
            .iter()
            .collect::<Option<BoundingBox>>()
            .expect("no lines");
        // Increase horizontal bounds to ensure that the left and right edges
        // never have clay.
        bounds.horizontal.start -= 1;
        bounds.horizontal.end += 1;
        // Make certain assumptions about our bounds explicit.
        assert!(bounds.horizontal.start <= 500);
        assert!(bounds.horizontal.end > 500);
        assert!(bounds.horizontal.start > 0);
        assert!(bounds.vertical.start > 0);
        let source = Point {
            x: 500,
            y: bounds.vertical.start - 1,
        };
        let tiles = vec![Tile::Sand; bounds.width() as usize * bounds.height() as usize];
        let mut map = Map {
            bounds,
            source,
            tiles,
        };
        for line in lines {
            map.draw(line, Tile::Clay);
        }
        map
    }

    fn draw(&mut self, line: &Line, tile: Tile) {
        match line {
            Line::Horizontal { x, y } => {
                for x in x.clone() {
                    self[Point { x, y: *y }] = tile;
                }
            },
            Line::Vertical { x, y } => {
                for y in y.clone() {
                    self[Point { x: *x, y }] = tile;
                }
            },
        }
    }

    pub fn get(&self, idx: Point) -> Option<&Tile> {
        let i = self.point_to_index(idx)?;
        Some(&self.tiles[i])
    }

    pub fn get_mut(&mut self, idx: Point) -> Option<&mut Tile> {
        let i = self.point_to_index(idx)?;
        Some(&mut self.tiles[i])
    }

    fn point_to_index(&self, idx: Point) -> Option<usize> {
        self.bounds
            .get_relative(idx)
            .map(|p| p.y * self.bounds.width() + p.x)
    }

    pub fn add_water(&mut self) -> WaterSummary {
        use self::FlowDir::*;
        enum Source {
            Head(Point),
            Foot(Point, usize),
        }
        assert_eq!(self[self.source.below()], Tile::Sand);
        let mut summary = WaterSummary::default();
        let mut stack = Vec::with_capacity(256);
        stack.push(Source::Head(self.source));
        while let Some(source) = stack.last() {
            match source {
                Source::Head(head) => match self.search(*head, Down) {
                    // There is clay or still water below.
                    SearchResult::Stopped(count) => {
                        if let Some(max_rise) = count.checked_sub(1) {
                            // The water can fall -- handle horizontal spill by
                            // pushing a new source. Mind the difference
                            // between `count` and `fall` here!
                            let foot = head.stepn(count, Down);
                            stack.push(Source::Foot(foot, max_rise));
                        } else {
                            // This top source is directly on top of an
                            // obstacle. It can no longer fall and hence can be
                            // removed.
                            stack.pop();
                        }
                    },
                    SearchResult::End(count) => {
                        summary.num_flowing += count;
                        let start = head.below();
                        let line = Line::Vertical {
                            x: start.x,
                            y: start.y..start.y + count,
                        };
                        self.draw(&line, Tile::FlowingWater);
                        stack.pop();
                    },
                    SearchResult::Overflow(_) => unreachable!("overflow on top source"),
                },
                Source::Foot(foot, max_rise) => {
                    match (self.search(*foot, Left), self.search(*foot, Right)) {
                        // Any overflow has to be handled first.
                        (SearchResult::Overflow(l), SearchResult::Overflow(r)) => {
                            let l = Source::Head(foot.stepn(l, Left));
                            let r = Source::Head(foot.stepn(r, Right));
                            stack.push(l);
                            stack.push(r);
                        },
                        (SearchResult::Overflow(l), _) => {
                            stack.push(Source::Head(foot.stepn(l, Left)));
                        },
                        (_, SearchResult::Overflow(r)) => {
                            stack.push(Source::Head(foot.stepn(r, Right)));
                        },
                        // Still water: Fill this line, put a new source on top
                        // and handle that in the next iteration. Never go
                        // beyond the original top source.
                        (SearchResult::Stopped(l), SearchResult::Stopped(r)) => {
                            summary.num_still += l + r + 1;
                            let line = Line::Horizontal {
                                x: foot.x - l..foot.x + r + 1,
                                y: foot.y,
                            };
                            self.draw(&line, Tile::StillWater);
                            if let Some(max_rise) = max_rise.checked_sub(1) {
                                let replacement = Source::Foot(foot.above(), max_rise);
                                stack.pop();
                                stack.push(replacement);
                            } else {
                                stack.pop();
                            }
                        },
                        // Water flows over and we have handled that overflow
                        // already. Just draw the horizontally flowing water.
                        (SearchResult::End(l), SearchResult::Stopped(r))
                        | (SearchResult::Stopped(l), SearchResult::End(r))
                        | (SearchResult::End(l), SearchResult::End(r)) => {
                            summary.num_flowing += l + r + 1;
                            let line = Line::Horizontal {
                                x: foot.x - l..foot.x + r + 1,
                                y: foot.y,
                            };
                            self.draw(&line, Tile::FlowingWater);
                            stack.pop();
                        },
                    }
                },
            }
        }
        summary
    }

    pub fn search(&self, start: Point, dir: FlowDir) -> SearchResult {
        match dir {
            FlowDir::Down => {
                let mut count = 0;
                let mut point = start;
                loop {
                    match self.get(point.below()) {
                        Some(Tile::Sand) => {
                            point = point.below();
                            count += 1;
                        },
                        Some(Tile::StillWater) | Some(Tile::Clay) => {
                            break SearchResult::Stopped(count)
                        },
                        Some(Tile::FlowingWater) | None => break SearchResult::End(count),
                    }
                }
            },
            dir @ FlowDir::Left | dir @ FlowDir::Right => {
                let mut count = 0;
                let mut point = start;
                loop {
                    match self[point.below()] {
                        Tile::Sand => break SearchResult::Overflow(count),
                        Tile::FlowingWater => break SearchResult::End(count),
                        Tile::StillWater | Tile::Clay => {},
                    }
                    if let Tile::Sand = self[point.step(dir)] {
                        point = point.step(dir);
                        count += 1;
                    } else {
                        break SearchResult::Stopped(count);
                    }
                }
            },
        }
    }
}

impl std::ops::Index<Point> for Map {
    type Output = Tile;

    fn index(&self, idx: Point) -> &Self::Output {
        let i = self
            .point_to_index(idx)
            .unwrap_or_else(|| panic!("out of bounds access: {:?} in {:?}", idx, self.bounds));
        &self.tiles[i]
    }
}

impl std::ops::IndexMut<Point> for Map {
    fn index_mut(&mut self, idx: Point) -> &mut Self::Output {
        let i = self
            .point_to_index(idx)
            .unwrap_or_else(|| panic!("out of bounds access: {:?} in {:?}", idx, self.bounds));
        &mut self.tiles[i]
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        for line in self.tiles.chunks(self.bounds.width()) {
            for tile in line {
                f.write_char(tile.to_char())?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

fn main() {
    let lines = {
        let stdin = io::stdin();
        stdin
            .lock()
            .lines()
            .map(|line| line.unwrap().trim().parse())
            .collect::<Result<Vec<Line>, _>>()
            .unwrap()
    };
    let mut map = Map::new(&lines);
    let summary = map.add_water();
    println!("lines: {}", lines.len());
    println!("bounds: {:?}", map.bounds);
    println!(
        "reachable by water: {}",
        summary.num_still + summary.num_flowing,
    );
    println!("remaining after flow stops: {}", summary.num_still);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basin() {
        let mut map = Map::new(&[
            Line::Horizontal { y: 10, x: 490..510 },
            Line::Vertical { x: 489, y: 8..11 },
            Line::Vertical { x: 510, y: 7..11 },
        ]);
        assert_eq!(
            map.bounds,
            BoundingBox {
                horizontal: 488..512,
                vertical: 7..11,
            },
        );
        assert_eq!(
            map.add_water(),
            WaterSummary {
                num_still: 40,
                num_flowing: 25
            },
        );
    }

    #[test]
    fn test_example() {
        let lines = "
x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504
";
        let lines = lines
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.parse())
            .collect::<Result<Vec<Line>, _>>()
            .unwrap();
        let mut map = Map::new(&lines);
        assert_eq!(
            map.bounds,
            BoundingBox {
                horizontal: 494..508,
                vertical: 1..14,
            },
        );
        assert_eq!(
            map.add_water(),
            WaterSummary {
                num_still: 29,
                num_flowing: 57 - 29
            },
        );
    }
}
