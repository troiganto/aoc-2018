use std::{
    cmp::max,
    io::{self, BufRead},
    num::ParseIntError,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone)]
struct Map<T> {
    contents: Vec<T>,
    width: usize,
    height: usize,
}

impl<T: Default + Clone> Map<T> {
    fn new(width: usize, height: usize) -> Self {
        Map {
            width,
            height,
            contents: vec![Default::default(); width * height],
        }
    }

    fn cells(&self) -> &[T] {
        &self.contents
    }

    #[allow(dead_code)]
    fn with_rectangle<F: FnMut(&T)>(&self, rect: &Rectangle, mut f: F) {
        for y in 0..rect.height {
            for x in 0..rect.width {
                let coord = (rect.x + x, rect.y + y);
                f(&self[coord])
            }
        }
    }

    fn with_rectangle_mut<F: FnMut(&mut T)>(&mut self, rect: &Rectangle, mut f: F) {
        for y in 0..rect.height {
            for x in 0..rect.width {
                let coord = (rect.x + x, rect.y + y);
                f(&mut self[coord])
            }
        }
    }
}

impl<T> Index<(usize, usize)> for Map<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        assert!(x < self.width && y < self.height);
        &self.contents[y * self.width + x]
    }
}

impl<T> IndexMut<(usize, usize)> for Map<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        assert!(x < self.width && y < self.height);
        &mut self.contents[y * self.width + x]
    }
}


#[derive(Debug, Clone)]
struct Rectangle {
    id: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone)]
struct NotARectangle;

impl From<ParseIntError> for NotARectangle {
    fn from(_: ParseIntError) -> Self {
        NotARectangle
    }
}


impl std::str::FromStr for Rectangle {
    type Err = NotARectangle;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = match s.split_at(1) {
            ("#", s) => s,
            _ => return Err(NotARectangle),
        };
        let pos = s.find(" @ ").ok_or(NotARectangle)?;
        let (id, s) = (s[..pos].parse()?, &s[pos + 3..]);
        let pos = s.find(",").ok_or(NotARectangle)?;
        let (x, s) = (s[..pos].parse()?, &s[pos + 1..]);
        let pos = s.find(": ").ok_or(NotARectangle)?;
        let (y, s) = (s[..pos].parse()?, &s[pos + 2..]);
        let pos = s.find("x").ok_or(NotARectangle)?;
        let (width, height) = (s[..pos].parse()?, s[pos + 1..].parse()?);
        let rect = Rectangle {
            id,
            x,
            y,
            width,
            height,
        };
        Ok(rect)
    }
}

fn main() {
    let stdin = io::stdin();
    let rectangles = stdin
        .lock()
        .lines()
        .map(Result::unwrap)
        .map(|line| line.parse())
        .collect::<Result<Vec<Rectangle>, NotARectangle>>()
        .unwrap();
    let (width, height) = rectangles.iter().fold((0, 0), |(width, height), rect| {
        (
            max(width, rect.x + rect.width),
            max(height, rect.y + rect.height),
        )
    });
    let mut map = Map::<u16>::new(width, height);
    for rect in &rectangles {
        map.with_rectangle_mut(rect, |count| *count += 1);
    }
    let overlapped_area = map.cells().iter().cloned().filter(|&x| x > 1).count();
    println!("overlapped area: {}", overlapped_area);
}
