use boolinator::Boolinator;
use std::{
    io::{self, BufRead},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Debug)]
enum Error {
    MissingCoord,
    TooManyCoords,
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}

#[derive(Debug, Default, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
    z: i32,
    t: i32,
}

impl Point {
    fn distance(&self, other: &Self) -> i32 {
        (self.x - other.x).abs()
            + (self.y - other.y).abs()
            + (self.z - other.z).abs()
            + (self.t - other.t).abs()
    }

    fn min_distance(&self, other: &[Self]) -> Option<i32> {
        other.iter().map(|s| self.distance(s)).min()
    }
}

impl FromStr for Point {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split(',');
        let x = words.next().ok_or(Error::MissingCoord)?.parse()?;
        let y = words.next().ok_or(Error::MissingCoord)?.parse()?;
        let z = words.next().ok_or(Error::MissingCoord)?.parse()?;
        let t = words.next().ok_or(Error::MissingCoord)?.parse()?;
        words.next().is_none().ok_or(Error::TooManyCoords)?;
        Ok(Point { x, y, z, t })
    }
}

struct Constellations(Vec<Point>);

impl Iterator for Constellations {
    type Item = Vec<Point>;

    fn next(&mut self) -> Option<Self::Item> {
        let seed = self.0.pop()?;
        let mut stars = vec![seed];
        while let Some(i) = self
            .0
            .iter()
            .position(|s| s.min_distance(&stars).unwrap() <= 3)
        {
            stars.push(self.0.swap_remove(i));
        }
        Some(stars)
    }
}

fn main() {
    let points = io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().trim().parse())
        .collect::<Result<Vec<Point>, _>>()
        .unwrap();
    println!("constellations: {}", Constellations(points).count());
}
