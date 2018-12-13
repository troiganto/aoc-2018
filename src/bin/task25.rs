use std::{
    collections::{BinaryHeap, HashMap},
    fmt::{self, Display},
    io::{self, Read},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Turn {
    Left,
    Straight,
    Right,
}

impl Turn {
    pub fn next(self) -> Self {
        match self {
            Turn::Left => Turn::Straight,
            Turn::Straight => Turn::Right,
            Turn::Right => Turn::Left,
        }
    }
}

impl Default for Turn {
    fn default() -> Self {
        Turn::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn turn(self, turn: Turn) -> Self {
        match turn {
            Turn::Left => self.turn_left(),
            Turn::Straight => self,
            Turn::Right => self.turn_right(),
        }
    }

    pub fn new(byte: u8) -> Result<Self, BadDirection> {
        match byte {
            b'^' => Ok(Direction::Up),
            b'>' => Ok(Direction::Right),
            b'<' => Ok(Direction::Left),
            b'v' => Ok(Direction::Down),
            _ => Err(BadDirection),
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Direction::Up => '^',
            Direction::Right => '>',
            Direction::Left => '<',
            Direction::Down => 'v',
        }
    }
}

#[derive(Debug, Clone)]
pub struct BadDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    // `y` comes first so that `PartialOrd` sorts by rows first.
    y: u32,
    x: u32,
}

impl Position {
    fn new(x: u32, y: u32) -> Self {
        Position { x, y }
    }

    fn step(self, dir: Direction) -> Self {
        let Position { mut x, mut y } = self;
        match dir {
            Direction::Up => y -= 1,
            Direction::Left => x -= 1,
            Direction::Right => x += 1,
            Direction::Down => y += 1,
        }
        Position { x, y }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BendType {
    LeftDownRightUp = b'\\',
    LeftUpRightDown = b'/',
    Intersection = b'+',
}

impl BendType {
    pub fn new(byte: u8) -> Result<Self, BadBend> {
        match byte {
            b'\\' => Ok(BendType::LeftDownRightUp),
            b'/' => Ok(BendType::LeftUpRightDown),
            b'+' => Ok(BendType::Intersection),
            _ => Err(BadBend),
        }
    }

    pub fn to_char(self) -> char {
        match self {
            BendType::LeftDownRightUp => '\\',
            BendType::LeftUpRightDown => '/',
            BendType::Intersection => '+',
        }
    }

    pub fn turn(&self, dir: Direction, turn: Turn) -> (Direction, Turn) {
        match self {
            BendType::LeftDownRightUp => match dir {
                Direction::Up => (Direction::Left, turn),
                Direction::Right => (Direction::Down, turn),
                Direction::Left => (Direction::Up, turn),
                Direction::Down => (Direction::Right, turn),
            },
            BendType::LeftUpRightDown => match dir {
                Direction::Up => (Direction::Right, turn),
                Direction::Right => (Direction::Up, turn),
                Direction::Left => (Direction::Down, turn),
                Direction::Down => (Direction::Left, turn),
            },
            BendType::Intersection => (dir.turn(turn), turn.next()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BadBend;

pub type Map = HashMap<Position, BendType>;

#[derive(Debug, Clone)]
pub struct Chart {
    pos: Position,
    dir: Direction,
    next_turn: Turn,
}

impl Chart {
    pub fn new(pos: Position, dir: Direction) -> Self {
        Chart {
            pos,
            dir,
            next_turn: Turn::default(),
        }
    }

    pub fn step(&mut self) {
        self.pos = self.pos.step(self.dir);
    }

    pub fn react_to_bend(&mut self, bend: BendType) {
        let (dir, turn) = bend.turn(self.dir, self.next_turn);
        self.dir = dir;
        self.next_turn = turn;
    }

    pub fn collides_with(&self, other: &Self) -> bool {
        self.pos == other.pos && self.dir != other.dir
    }
}

pub fn read_map(bytes: &[u8]) -> (Map, Vec<Chart>) {
    let mut pos = Position::new(0, 0);
    let mut map = Map::new();
    let mut charts = Vec::new();
    for &byte in bytes {
        // Read data.
        if let Ok(bend) = BendType::new(byte) {
            map.insert(pos, bend);
        } else if let Ok(dir) = Direction::new(byte) {
            charts.push(Chart::new(pos, dir));
        }
        // Move cursor.
        if byte == b'\n' {
            pos.x = 0;
            pos.y += 1
        } else {
            pos.x += 1;
        }
    }
    (map, charts)
}

pub struct MinPosOrder(pub Chart);

impl std::cmp::Ord for MinPosOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        std::cmp::Ord::cmp(&self.0.pos, &other.0.pos).reverse()
    }
}

impl std::cmp::PartialOrd for MinPosOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for MinPosOrder {
    fn eq(&self, other: &Self) -> bool {
        self.0.pos == other.0.pos
    }
}

impl std::cmp::Eq for MinPosOrder {}

pub fn simulate_until_first_collision(charts: Vec<Chart>, map: &Map) -> Position {
    let mut charts = charts
        .into_iter()
        .map(MinPosOrder)
        .collect::<BinaryHeap<_>>();
    let mut next_charts = BinaryHeap::with_capacity(charts.len());
    loop {
        while let Some(MinPosOrder(mut chart)) = charts.pop() {
            chart.step();
            if let Some(&bend) = map.get(&chart.pos) {
                chart.react_to_bend(bend);
            }
            if let Some(pos) = charts
                .iter()
                .chain(next_charts.iter())
                .find(|MinPosOrder(c)| c.collides_with(&chart))
                .map(|MinPosOrder(Chart { pos, .. })| *pos)
            {
                return pos;
            }
            next_charts.push(MinPosOrder(chart));
        }
        std::mem::swap(&mut charts, &mut next_charts);
    }
}

pub fn simulate_until_one_chart_left(charts: Vec<Chart>, map: &Map) -> Position {
    let mut charts = charts.into_iter().map(Some).collect::<Vec<_>>();
    loop {
        charts.sort_unstable_by_key(|c| c.as_ref().unwrap().pos);
        for i in 0..charts.len() {
            // Update the current chart's position.
            if let Some(chart) = &mut charts[i] {
                chart.step();
                if let Some(&bend) = map.get(&chart.pos) {
                    chart.react_to_bend(bend);
                }
            }
            // Find a collision and mark both charts as dead if there is one.
            let j = charts[i].as_ref().and_then(|chart| {
                charts.iter().position(|c| match c {
                    Some(c) => c.collides_with(chart),
                    None => false,
                })
            });
            if let Some(j) = j {
                charts[i] = None;
                charts[j] = None;
            }
        }
        // Remove all dead charts. If there is one chart left, return it.
        charts.retain(|chart| chart.is_some());
        if charts.len() == 1 {
            return charts[0].as_ref().unwrap().pos;
        }
    }
}

fn main() {
    let (map, charts) = {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap();
        read_map(&buf)
    };
    let collision = simulate_until_first_collision(charts.clone(), &map);
    println!("first collision: {}", collision);
    let winner = simulate_until_one_chart_left(charts, &map);
    println!("last chart: {}", winner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_chart_left() {
        let (map, charts) = read_map(
            br"/>-<\
|   |
| /<+-\
| | | v
\>+</ |
  |   ^
  \<->/
",
        );
        let winner = simulate_until_one_chart_left(charts, &map);
        assert_eq!(winner, Position { x: 6, y: 4 });
    }
}
