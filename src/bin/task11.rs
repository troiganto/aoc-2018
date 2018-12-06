use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::{self, BufRead},
    num::ParseIntError,
    ops::{Add, Sub},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn distance(&self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn is_on_edge(&self, board: &Board) -> bool {
        self.x == board.top_left.x
            || self.y == board.top_left.y
            || self.x == board.bottom_right.x
            || self.y == board.bottom_right.y
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Debug)]
pub enum ParsePointError {
    ParseIntError(ParseIntError),
    BadCoords,
}

impl From<ParseIntError> for ParsePointError {
    fn from(err: ParseIntError) -> Self {
        ParsePointError::ParseIntError(err)
    }
}

impl FromStr for Point {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::ParsePointError::*;

        let mut coords = s.splitn(2, ",");
        let x = coords.next().ok_or(BadCoords)?.trim_end().parse()?;
        let y = coords.next().ok_or(BadCoords)?.trim_start().parse()?;
        debug_assert!(coords.next().is_none());
        Ok(Point { x, y })
    }
}

impl Add<(i32, i32)> for Point {
    type Output = Point;

    fn add(self, (dx, dy): (i32, i32)) -> Self::Output {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl Sub<(i32, i32)> for Point {
    type Output = Point;

    fn sub(self, (dx, dy): (i32, i32)) -> Self::Output {
        Point {
            x: self.x - dx,
            y: self.y - dy,
        }
    }
}

struct Board {
    top_left: Point,
    bottom_right: Point,
}

impl Board {
    fn new(init: Point) -> Board {
        Board {
            top_left: Point {
                x: init.x - 1,
                y: init.y - 1,
            },
            bottom_right: Point {
                x: init.x + 1,
                y: init.y + 1,
            },
        }
    }

    fn extend_to(&mut self, p: Point) {
        use std::cmp;
        self.top_left.x = cmp::min(self.top_left.x, p.x - 1);
        self.top_left.y = cmp::min(self.top_left.y, p.y - 1);
        self.bottom_right.x = cmp::max(self.bottom_right.x, p.x + 1);
        self.bottom_right.y = cmp::max(self.bottom_right.y, p.y + 1);
    }

    fn iter(&self) -> impl Iterator<Item = Point> {
        let (min, max) = (self.top_left, self.bottom_right);
        (min.x..=max.x).flat_map(move |x| (min.y..=max.y).map(move |y| Point { x, y }))
    }
}

impl Extend<Point> for Board {
    fn extend<T>(&mut self, points: T)
    where
        T: IntoIterator<Item = Point>,
    {
        for point in points {
            self.extend_to(point);
        }
    }
}

impl std::iter::FromIterator<Point> for Option<Board> {
    fn from_iter<T>(points: T) -> Self
    where
        T: IntoIterator<Item = Point>,
    {
        let mut points = points.into_iter();
        let mut board = Board::new(points.next()?);
        board.extend(points);
        Some(board)
    }
}

fn find_nearest_point(refp: Point, candidates: impl IntoIterator<Item = Point>) -> Option<Point> {
    let mut nearest = None;
    let mut best_distance = None;
    for candidate in candidates {
        let distance = candidate.distance(refp);
        match (distance, best_distance) {
            (d, None) => {
                // No best distance yet, automatic win.
                nearest = Some(candidate);
                best_distance = Some(d);
            },
            (d, Some(bd)) if d < bd => {
                // Better than the previously best candidate, replace it.
                nearest = Some(candidate);
                best_distance = Some(d);
            },
            (d, Some(bd)) if d == bd => {
                // Tie, no one wins, but the next candidate must still
                // be better than the currently best distance.
                nearest = None;
            },
            _ => {},
        }
    }
    nearest
}


fn find_largest_area(coords: impl IntoIterator<Item = Point>) -> Option<(Point, usize)> {
    let mut coords = coords
        .into_iter()
        .map(|p| (p, Some(0)))
        .collect::<HashMap<Point, Option<usize>>>();
    let board = coords.keys().cloned().collect::<Option<Board>>().unwrap();
    for point in board.iter() {
        if let Some(coord) = find_nearest_point(point, coords.keys().cloned()) {
            if point.is_on_edge(&board) {
                *coords.get_mut(&coord).unwrap() = None;
            } else if let Some(counter) = coords.get_mut(&coord).unwrap() {
                *counter += 1;
            }
        }
    }
    coords
        .into_iter()
        .filter_map(|(p, c)| c.map(|c| (p, c)))
        .max_by_key(|&(_, c)| c)
}

fn find_area_within_total_distance(coords: &[Point], max_distance: i32) -> usize {
    let mut board = coords.iter().cloned().collect::<Option<Board>>().unwrap();
    board.extend_to(board.top_left - (max_distance, max_distance));
    board.extend_to(board.bottom_right + (max_distance, max_distance));
    board
        .iter()
        .filter(|&p| coords.iter().map(|&c| p.distance(c)).sum::<i32>() < max_distance)
        .count()
}

fn main() {
    let stdin = io::stdin();
    let coords = stdin
        .lock()
        .lines()
        .map(|line| line.unwrap().trim().parse())
        .collect::<Result<Vec<Point>, _>>()
        .unwrap();
    let (maxp, max) = find_largest_area(coords.iter().cloned()).unwrap();
    println!("best point {} (area = {})", maxp, max);
    let min_area = find_area_within_total_distance(&coords, 10000);
    println!("minimal area: {}", min_area);
}
