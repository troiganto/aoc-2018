use std::{
    io::{self, BufRead},
    ops::{Add, AddAssign, Sub},
    str::FromStr,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point {
    pub pos: Position,
    pub vel: Velocity,
}

impl Point {
    pub fn neighbors(&self, other: &Self) -> bool {
        let diff = self.pos.clone() - other.pos.clone();
        [
            Velocity { x: 1, y: 0 },
            Velocity { x: -1, y: 0 },
            Velocity { x: 0, y: 1 },
            Velocity { x: 0, y: -1 },
        ]
        .contains(&diff)
    }

    pub fn step(&mut self) {
        self.pos += &self.vel;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Velocity {
    x: i32,
    y: i32,
}

impl Add<Velocity> for Position {
    type Output = Self;

    fn add(self, other: Velocity) -> Self::Output {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<'a> Add<&'a Velocity> for Position {
    type Output = Self;

    fn add(self, other: &'a Velocity) -> Self::Output {
        self + other.clone()
    }
}

impl AddAssign<Velocity> for Position {
    fn add_assign(&mut self, other: Velocity) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<'a> AddAssign<&'a Velocity> for Position {
    fn add_assign(&mut self, other: &'a Velocity) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Position {
    type Output = Velocity;

    fn sub(self, other: Self) -> Self::Output {
        Velocity {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParsePointError {
    NoPosition,
    NoVelocity,
    NoParens,
    NoX,
    NoY,
    TooManyCoords,
    ParseIntError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for ParsePointError {
    fn from(err: std::num::ParseIntError) -> Self {
        ParsePointError::ParseIntError(err)
    }
}

impl FromStr for Position {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use boolinator::Boolinator;

        let split = s.find("=").ok_or(ParsePointError::NoPosition)?;
        let (left, s) = s.split_at(split + 1);
        (left == "position=").ok_or(ParsePointError::NoPosition)?;
        (s.starts_with("<") && s.ends_with(">")).ok_or(ParsePointError::NoParens)?;
        let s = &s[1..s.len() - 1];
        let mut s = s.splitn(2, ",");
        let x = s.next().ok_or(ParsePointError::NoX)?.trim().parse()?;
        let y = s.next().ok_or(ParsePointError::NoY)?.trim().parse()?;
        s.next().is_none().ok_or(ParsePointError::TooManyCoords)?;
        Ok(Position { x, y })
    }
}

impl FromStr for Velocity {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use boolinator::Boolinator;

        let split = s.find("=").ok_or(ParsePointError::NoVelocity)?;
        let (left, s) = s.split_at(split + 1);
        (left == "velocity=").ok_or(ParsePointError::NoVelocity)?;
        (s.starts_with("<") && s.ends_with(">")).ok_or(ParsePointError::NoParens)?;
        let s = &s[1..s.len() - 1];
        let mut s = s.splitn(2, ",");
        let x = s.next().ok_or(ParsePointError::NoX)?.trim().parse()?;
        let y = s.next().ok_or(ParsePointError::NoY)?.trim().parse()?;
        s.next().is_none().ok_or(ParsePointError::TooManyCoords)?;
        Ok(Velocity { x, y })
    }
}

impl FromStr for Point {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.find(">").ok_or(ParsePointError::NoPosition)?;
        let (pos, vel) = s.split_at(split + 1);
        let pos = pos.trim().parse()?;
        let vel = vel.trim().parse()?;
        Ok(Point { pos, vel })
    }
}

fn are_aligned(stars: &[Point]) -> bool {
    let threshold = stars.len() * 3 / 20;
    let lone_stars = stars
        .iter()
        .filter(|l| stars.iter().filter(|r| l.neighbors(r)).nth(1).is_none());
    { lone_stars }.nth(threshold).is_none()
}

fn draw(points: &[Point]) {
    let top_left = Position {
        x: points.iter().map(|p| p.pos.x).min().unwrap(),
        y: points.iter().map(|p| p.pos.y).min().unwrap(),
    };
    let bottom_right = Position {
        x: 1 + points.iter().map(|p| p.pos.x).max().unwrap(),
        y: 1 + points.iter().map(|p| p.pos.y).max().unwrap(),
    };
    let Velocity {
        x: width,
        y: height,
    } = bottom_right.clone() - top_left.clone();
    assert!(width > 0);
    assert!(height > 0);
    assert!(width < 4000);
    assert!(height < 4000);
    let mut buf = image::GrayImage::new(width as u32, height as u32);
    for point in points {
        let Velocity { x, y } = point.pos.clone() - top_left.clone();
        assert!(x >= 0);
        assert!(y >= 0);
        assert!(x < width);
        assert!(y < height);
        *buf.get_pixel_mut(x as u32, y as u32) = image::Luma([u8::max_value()]);
    }
    buf.save("part1.png").unwrap();
}

fn main() {
    let stdin = io::stdin();
    let mut points = stdin
        .lock()
        .lines()
        .map(|line| line.unwrap().parse())
        .collect::<Result<Vec<Point>, _>>()
        .unwrap();
    let mut seconds = 0;
    while !are_aligned(&points) {
        points.iter_mut().for_each(Point::step);
        seconds += 1;
    }
    println!("simulation done after {} secs", seconds);
    draw(&points);
}
