use boolinator::Boolinator;
use std::{
    borrow::Borrow,
    fmt,
    io::{self, BufRead},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Debug)]
enum ParseError {
    UnknownProperty,
    MissingParens,
    MissingRadius,
    MissingPoint,
    TooFewCoords,
    TooManyCoords,
    TooManyRadii,
    TooManyPoints,
    BadRadius,
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::ParseIntError(err)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct Point {
    x: i64,
    y: i64,
    z: i64,
}

impl Point {
    fn distance(&self, other: &Self) -> i64 {
        (other.x - self.x).abs() + (other.y - self.y).abs() + (other.z - self.z).abs()
    }
}

impl FromStr for Point {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.starts_with('<').ok_or(ParseError::MissingParens)?;
        s.ends_with('>').ok_or(ParseError::MissingParens)?;
        let s = &s[1..s.len() - 1];
        let mut words = s.split(',');
        let x = words.next().ok_or(ParseError::TooFewCoords)?.parse()?;
        let y = words.next().ok_or(ParseError::TooFewCoords)?.parse()?;
        let z = words.next().ok_or(ParseError::TooFewCoords)?.parse()?;
        words.next().is_none().ok_or(ParseError::TooManyCoords)?;
        Ok(Point { x, y, z })
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{},{},{}>", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct Bot {
    pos: Point,
    radius: i64,
}

impl Bot {
    fn is_in_range(&self, pos: &Point) -> bool {
        self.pos.distance(pos) <= self.radius
    }
}
impl FromStr for Bot {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pos = None;
        let mut radius = None;
        for word in s.split(", ") {
            if word.starts_with("pos=") {
                pos.is_none().ok_or(ParseError::TooManyPoints)?;
                pos = Some(word[4..].parse()?);
            } else if word.starts_with("r=") {
                radius.is_none().ok_or(ParseError::TooManyRadii)?;
                radius = Some(word[2..].parse()?);
            } else {
                Err(ParseError::UnknownProperty)?;
            }
        }
        let pos = pos.ok_or(ParseError::MissingPoint)?;
        let radius = radius.ok_or(ParseError::MissingRadius)?;
        (radius > 0).ok_or(ParseError::BadRadius)?;
        Ok(Bot { pos, radius })
    }
}

fn count_bots_in_range<I>(bots: I, pos: &Point) -> usize
where
    I: IntoIterator,
    I::Item: Borrow<Bot>,
{
    bots.into_iter()
        .filter(|bot| bot.borrow().is_in_range(pos))
        .count()
}

fn count_points_in_range<I>(points: I, bot: &Bot) -> usize
where
    I: IntoIterator,
    I::Item: Borrow<Point>,
{
    points
        .into_iter()
        .filter(|p| bot.is_in_range(p.borrow()))
        .count()
}

fn main() {
    let bots = io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap().trim_end().parse())
        .collect::<Result<Vec<Bot>, _>>()
        .unwrap();
    let strongest = bots.iter().max_by_key(|bot| bot.radius).cloned().unwrap();
    println!("strongest bot: {:?}", strongest);
    println!(
        "in range: {}",
        count_points_in_range(bots.iter().map(|bot| bot.pos), &strongest),
    );
    let mut c = bots
        .iter()
        .map(|bot| count_bots_in_range(&bots, &bot.pos))
        .collect::<Vec<_>>();
    c.sort();
    c.reverse();
    for i in &c[c[0]..] {
        println!("{}", i);
    }
}
