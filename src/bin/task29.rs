use std::{
    collections::{HashMap, HashSet},
    fmt,
    io::{self, Read},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Tile {
    Empty = b'.',
    Wall = b'#',
    Gnome = b'G',
    Elf = b'E',
}

impl Tile {
    fn is_team(&self, team: Team) -> bool {
        match (self, team) {
            (Tile::Gnome, Team::Gnome) => true,
            (Tile::Elf, Team::Elf) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
enum Direction {
    Up,
    Left,
    Right,
    Down,
}

#[must_use]
fn directions() -> Directions {
    Directions(Some(Direction::Up))
}

#[derive(Debug, Clone)]
struct Directions(Option<Direction>);

impl Iterator for Directions {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        use self::Direction::*;
        let result = self.0?;
        self.0 = match result {
            Up => Some(Left),
            Left => Some(Right),
            Right => Some(Down),
            Down => None,
        };
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for Directions {
    fn len(&self) -> usize {
        use self::Direction::*;
        match self.0 {
            Some(Up) => 4,
            Some(Left) => 3,
            Some(Right) => 2,
            Some(Down) => 1,
            None => 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Point {
    y: i16,
    x: i16,
}

impl Point {
    fn new(x: i16, y: i16) -> Self {
        Point { x, y }
    }

    #[must_use]
    fn step(self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y + 1,
            },
        }
    }

    fn neighbors(&self) -> Neighbors {
        Neighbors::new(*self)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone)]
struct Neighbors {
    center: Point,
    directions: Directions,
}

impl Neighbors {
    fn new(center: Point) -> Self {
        Neighbors {
            center,
            directions: directions(),
        }
    }
}

impl Iterator for Neighbors {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.directions.next().map(|d| self.center.step(d))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.directions.size_hint()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Team {
    Elf = b'E',
    Gnome = b'G',
}

impl Team {
    #[must_use]
    fn enemy(self) -> Self {
        use self::Team::*;
        match self {
            Elf => Gnome,
            Gnome => Elf,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Unit {
    team: Team,
    hp: u8,
    attack: u8,
}

impl Unit {
    fn take_damage(&mut self, hp: u8) {
        self.hp = self.hp.saturating_sub(hp);
    }

    fn is_defeated(&self) -> bool {
        self.hp == 0
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self.team {
            Team::Elf => 'E',
            Team::Gnome => 'G',
        };
        write!(f, "{}({})", c, self.hp)
    }
}

#[derive(Debug)]
struct Board {
    size: Point,
    map: Vec<Tile>,
    units: HashMap<Point, Unit>,
}

impl Board {
    fn new<T>(bytes: T) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        Self::with_elven_power(bytes, 3)
    }

    fn with_elven_power<T>(bytes: T, power: u8) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        let mut map = Vec::new();
        let mut pos = Point::new(0, 0);
        let mut units = HashMap::new();
        let mut size = Point::new(0, 0);
        for b in bytes {
            match b {
                b'.' => {
                    map.push(Tile::Empty);
                    pos.x += 1;
                },
                b'#' => {
                    map.push(Tile::Wall);
                    pos.x += 1;
                },
                b'G' => {
                    map.push(Tile::Gnome);
                    let old = units.insert(
                        pos,
                        Unit {
                            team: Team::Gnome,
                            hp: 200,
                            attack: 3,
                        },
                    );
                    debug_assert!(old.is_none());
                    pos.x += 1;
                },
                b'E' => {
                    map.push(Tile::Elf);
                    let old = units.insert(
                        pos,
                        Unit {
                            team: Team::Elf,
                            hp: 200,
                            attack: power,
                        },
                    );
                    debug_assert!(old.is_none());
                    pos.x += 1;
                },
                b'\n' => {
                    size = pos;
                    pos.x = 0;
                    pos.y += 1;
                },
                _ => panic!("unknown character: {}", b as char),
            }
        }
        Board { size, map, units }
    }

    fn bounds_check(&self, p: Point) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.size.x && p.y < self.size.y
    }

    fn get(&self, p: Point) -> Option<&Tile> {
        use boolinator::Boolinator;
        self.bounds_check(p).as_option()?;
        Some(&self.map[self.index_of_point(p)])
    }

    fn get_mut(&mut self, p: Point) -> Option<&mut Tile> {
        use boolinator::Boolinator;
        self.bounds_check(p).as_option()?;
        let index = self.index_of_point(p);
        Some(&mut self.map[index])
    }

    fn total_hp(&self) -> u64 {
        self.units.values().map(|Unit { hp, .. }| *hp as u64).sum()
    }

    fn team_won(&self, winner: Team) -> bool {
        self.units.values().all(|Unit { team, .. }| *team == winner)
    }

    fn index_of_point(&self, p: Point) -> usize {
        assert!(
            self.bounds_check(p),
            "point {} out of bounds for board of size {}",
            p,
            self.size,
        );
        (p.y as usize * self.size.x as usize) + p.x as usize
    }

    fn distance_to_closest_unit(&self, start: Point, goal: Team) -> Option<usize> {
        use std::collections::hash_map::Entry;

        #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
        struct Steps(usize);

        if self[start].is_team(goal) {
            // Short circuit logic.
            return Some(0);
        }
        let mut open = HashMap::new();
        open.insert(start, Steps(0));
        let mut visited = HashSet::new();
        while let Some((&current, &dist)) = open.iter().min_by_key(|&(point, dist)| (dist, point)) {
            if current
                .neighbors()
                .filter_map(|neighbor| self.get(neighbor))
                .any(|tile| tile.is_team(goal))
            {
                return Some(dist.0 + 1);
            }
            open.remove(&current);
            visited.insert(current);
            // Check all neighbors.
            // Ignore all occupied and visited neighbors.
            for neighbor in current
                .neighbors()
                .filter(|p| !visited.contains(p))
                .filter(|&p| self.get(p).cloned() == Some(Tile::Empty))
            {
                let next_dist = Steps(dist.0 + 1);
                match open.entry(neighbor) {
                    Entry::Vacant(entry) => {
                        entry.insert(next_dist);
                    },
                    Entry::Occupied(mut entry) => {
                        // Update the distance if the current one is better.
                        let old_dist = entry.get_mut();
                        *old_dist = std::cmp::min(*old_dist, next_dist);
                    },
                }
            }
        }
        None
    }

    fn find_enemy_direction(&self, start: Point) -> Option<Direction> {
        use boolinator::Boolinator;
        let team = self.units[&start].team;
        directions()
            .map(|dir| (dir, start.step(dir)))
            .filter_map(|(dir, p)| self.get(p).map(|&tile| (dir, p, tile)))
            .filter(|&(_, _, tile)| tile != Tile::Wall && !tile.is_team(team))
            .filter_map(|(dir, p, _)| {
                self.distance_to_closest_unit(p, team.enemy())
                    .map(|dist| (dir, dist))
            })
            .min_by_key(|&(_, dist)| dist)
            .and_then(|(dir, dist)| (dist > 0).as_some(dir))
    }

    fn move_unit(&mut self, pos: Point, dir: Direction) -> Point {
        let unit = self.units.remove(&pos).expect("no unit to move");
        let old_tile = match self.get_mut(pos) {
            Some(tile) => {
                assert!(tile.is_team(unit.team), "map and unit list are inconsisten");
                std::mem::replace(tile, Tile::Empty)
            },
            None => panic!("position {} out of bounds", pos),
        };
        let goal = pos.step(dir);
        self.units.insert(goal, unit);
        match self.get_mut(goal) {
            Some(tile) => {
                if *tile == Tile::Empty {
                    *tile = old_tile;
                } else {
                    panic!("move onto non-empty field at {}", goal);
                }
            },
            None => panic!("position {} out of bounds", pos),
        }
        goal
    }

    fn attack(&mut self, attacker: Point, attacked: Point) -> Option<Unit> {
        use std::collections::hash_map::Entry;
        // Access the attacked unit and deal damage.
        let damage = self.units[&attacker].attack;
        let mut unit = match self.units.entry(attacked) {
            Entry::Occupied(unit) => unit,
            _ => panic!("no unit to attack"),
        };
        unit.get_mut().take_damage(damage);
        // If the unit is defeated, remove it from map and unit list.
        let unit = if unit.get().is_defeated() {
            unit.remove()
        } else {
            return None;
        };
        let tile = self.get_mut(attacked).expect("attacked point out of range");
        assert!(
            tile.is_team(unit.team),
            "map and unit list are inconsistent",
        );
        *tile = Tile::Empty;
        Some(unit)
    }
}

impl std::ops::Index<Point> for Board {
    type Output = Tile;

    fn index(&self, index: Point) -> &Self::Output {
        if let Some(tile) = self.get(index) {
            tile
        } else {
            panic!("index {} out of bounds for board {}", index, self.size);
        }
    }
}

impl std::ops::IndexMut<Point> for Board {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        let size = self.size;
        if let Some(tile) = self.get_mut(index) {
            return tile;
        } else {
            panic!("index {} out of bounds for board {}", index, size);
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        for (i, line) in self.map.chunks(self.size.x as usize).enumerate() {
            for &tile in line {
                f.write_char(tile as u8 as char)?;
            }
            let mut units = self
                .units
                .iter()
                .filter(|(&Point { y, .. }, _)| y as usize == i)
                .collect::<Vec<_>>();
            units.sort_by_key(|(&p, _)| p);
            f.write_str("  ")?;
            for (_, unit) in units {
                write!(f, "{} ", unit)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Going,
    Over,
}

#[derive(Debug)]
struct Game {
    completed_turns: u64,
    to_be_moved: Vec<Point>,
    board: Board,
    defeated_elves: usize,
}

impl Game {
    fn new(board: &[u8]) -> Self {
        Board::new(board.iter().cloned()).into()
    }

    fn with_elven_power(board: &[u8], power: u8) -> Self {
        Board::with_elven_power(board.iter().cloned(), power).into()
    }
}

impl From<Board> for Game {
    fn from(board: Board) -> Self {
        let mut to_be_moved = board.units.keys().cloned().collect::<Vec<_>>();
        to_be_moved.sort_by_key(|&p| std::cmp::Reverse(p));
        Game {
            defeated_elves: 0,
            completed_turns: 0,
            to_be_moved,
            board,
        }
    }
}

impl Game {
    fn start_new_turn(&mut self) {
        self.completed_turns += 1;
        self.to_be_moved = self.board.units.keys().cloned().collect::<Vec<_>>();
        self.to_be_moved.sort_by_key(|&p| std::cmp::Reverse(p));
        assert!(!self.to_be_moved.is_empty(), "no units left");
    }


    fn progress(&mut self) -> GameState {
        // Step 1: Get the next unit to move.
        let this_pos = self.to_be_moved.pop().expect("no units lefts");
        let this_team = self.board.units[&this_pos].team;
        let enemy_team = this_team.enemy();
        // Step 2: Check if game is over.
        if self.board.team_won(this_team) {
            return GameState::Over;
        }
        // Step 3: Move.
        let this_pos = if let Some(dir) = self.board.find_enemy_direction(this_pos) {
            self.board.move_unit(this_pos, dir)
        } else {
            this_pos
        };
        // Step 4: Attack.
        // Find the adjacent enemy with the lowest HP (tie-break by position).
        if let Some(attacked) = this_pos
            .neighbors()
            .filter_map(|adjacent| self.board.units.get(&adjacent).map(|unit| (adjacent, unit)))
            .filter(|&(_, Unit { team, .. })| *team == enemy_team)
            .min_by_key(|&(point, Unit { hp, .. })| (hp, point))
            .map(|(point, _)| point)
        {
            let defeated = self.board.attack(this_pos, attacked);
            // If a unit has been defeated, remove it from the list of units to be moved.
            if let Some(unit) = defeated {
                if unit.team == Team::Elf {
                    self.defeated_elves += 1;
                }
                if let Ok(pos) = self
                    .to_be_moved
                    .binary_search_by_key(&std::cmp::Reverse(attacked), |&p| std::cmp::Reverse(p))
                {
                    self.to_be_moved.remove(pos);
                }
            }
        }
        // Step 5: Check if a turn has been completed.
        if self.to_be_moved.is_empty() {
            self.start_new_turn();
        }
        GameState::Going
    }

    fn make_turn(&mut self) -> GameState {
        let old = self.completed_turns;
        loop {
            let state = self.progress();
            if state == GameState::Over || self.completed_turns != old {
                return state;
            }
        }
    }

    fn checksum(&self) -> u64 {
        self.completed_turns * self.board.total_hp()
    }
}

fn main() {
    let mut board = Vec::new();
    io::stdin().read_to_end(&mut board).unwrap();
    let mut game = Game::new(&board);
    while let GameState::Going = game.make_turn() {}
    println!("completed turns: {}", game.completed_turns);
    println!("checksum: {}", game.checksum());
    if game.defeated_elves == 0 {
        println!("perfect game!");
    } else {
        for power in 4.. {
            let mut game = Game::with_elven_power(&board, power);
            while let GameState::Going = game.make_turn() {}
            if game.defeated_elves == 0 {
                println!("minimum power for perfect game: {}", power);
                println!("perfect checksum: {}", game.checksum());
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_units(game: &Game, expected: &[(i16, i16, Team, u8)]) {
        let mut expected = expected
            .iter()
            .map(|&(x, y, team, hp)| {
                let mut unit = Unit::new(team);
                unit.hp = hp;
                (Point { x, y }, unit)
            })
            .collect::<Vec<_>>();
        let mut actual = game.board.units.iter().collect::<Vec<_>>();
        expected.sort_by_key(|&(p, _)| p);
        actual.sort_by_key(|(&p, _)| p);
        assert_eq!(actual.len(), expected.len(), "actual len != expected len");
        for (i, ((lp, lu), (rp, ru))) in actual.into_iter().zip(expected.into_iter()).enumerate() {
            assert_eq!((lp, lu), (&rp, &ru), "mismatch on unit #{}", i);
        }
    }

    fn assert_game(board: &[u8], attack: u8, turns: u64, hp: u64, defeated_elves: usize) -> Game {
        let mut game = Game::with_elven_power(board, attack);
        while let GameState::Going = game.make_turn() {}
        assert_eq!(
            (game.completed_turns, game.checksum(), game.defeated_elves),
            (turns, turns * hp, defeated_elves),
        );
        game
    }

    #[test]
    fn test_find_steps() {
        let board = b"#######
#E..G.#
#...#.#
#.GE#G#
#######
";
        let board = Board::new(board.iter().cloned());
        // Find correct path among multiple equivalent ones.
        assert_eq!(
            board.find_enemy_direction(Point { x: 1, y: 1 }),
            Some(Direction::Right),
        );
        // Correctly detect unreachable point.
        assert_eq!(board.find_enemy_direction(Point { x: 5, y: 3 }), None);
        // Correctly detect already being at the goal.
        assert_eq!(board.find_enemy_direction(Point { x: 3, y: 3 }), None);
    }

    #[test]
    fn test_tricky_movement() {
        use self::Direction::*;
        let board = b"#######
#E....#
#G.E.E#
#######
";
        let board = Board::new(board.iter().cloned());
        assert_eq!(board.find_enemy_direction(Point { x: 1, y: 1 }), None);
        assert_eq!(board.find_enemy_direction(Point { x: 3, y: 2 }), Some(Left));
        assert_eq!(board.find_enemy_direction(Point { x: 5, y: 2 }), Some(Up));
    }

    #[test]
    fn test_diagonal_movement() {
        use self::Direction::*;
        let board = b"#####
#..G#
#...#
#...#
#E..#
#####
";
        let board = Board::new(board.iter().cloned());
        assert_eq!(board.find_enemy_direction(Point { x: 3, y: 1 }), Some(Left));
    }

    #[test]
    fn test_movement() {
        use self::Team::*;
        let mut game = Game::new(
            b"#########
#G..G..G#
#.......#
#.......#
#G..E..G#
#.......#
#.......#
#G..G..G#
#########
",
        );
        assert_units(
            &game,
            &[
                (1, 1, Gnome, 200),
                (4, 1, Gnome, 200),
                (7, 1, Gnome, 200),
                (1, 4, Gnome, 200),
                (4, 4, Elf, 200),
                (7, 4, Gnome, 200),
                (1, 7, Gnome, 200),
                (4, 7, Gnome, 200),
                (7, 7, Gnome, 200),
            ],
        );
        game.make_turn();
        assert_units(
            &game,
            &[
                (2, 1, Gnome, 200),
                (4, 2, Gnome, 197),
                (6, 1, Gnome, 200),
                (2, 4, Gnome, 200),
                (4, 3, Elf, 200),
                (7, 3, Gnome, 200),
                (1, 6, Gnome, 200),
                (4, 6, Gnome, 200),
                (7, 6, Gnome, 200),
            ],
        );
        game.make_turn();
        assert_units(
            &game,
            &[
                (3, 1, Gnome, 200),
                (4, 2, Gnome, 194),
                (5, 1, Gnome, 200),
                (2, 3, Gnome, 200),
                (4, 3, Elf, 197),
                (6, 3, Gnome, 200),
                (1, 5, Gnome, 200),
                (4, 5, Gnome, 200),
                (7, 5, Gnome, 200),
            ],
        );
        game.make_turn();
        assert_units(
            &game,
            &[
                (3, 2, Gnome, 200),
                (4, 2, Gnome, 191),
                (5, 2, Gnome, 200),
                (3, 3, Gnome, 200),
                (4, 3, Elf, 185),
                (5, 3, Gnome, 200),
                (1, 4, Gnome, 200),
                (4, 4, Gnome, 200),
                (7, 5, Gnome, 200),
            ],
        );
    }

    #[test]
    fn test_combat1() {
        use self::Team::*;
        let board = b"#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######
";
        let game = assert_game(board, 3, 47, 590, 2);
        assert_units(
            &game,
            &[
                (1, 1, Gnome, 200),
                (2, 2, Gnome, 131),
                (5, 3, Gnome, 59),
                (5, 5, Gnome, 200),
            ],
        );
        assert_game(board, 15, 29, 172, 0);
    }

    #[test]
    fn test_combat2() {
        use self::Team::*;
        let board = b"#######
#G..#E#
#E#E.E#
#G.##.#
#...#E#
#...E.#
#######
";
        let game = assert_game(board, 3, 37, 982, 1);
        assert_units(
            &game,
            &[
                (5, 1, Elf, 200),
                (1, 2, Elf, 197),
                (2, 3, Elf, 185),
                (1, 4, Elf, 200),
                (5, 4, Elf, 200),
            ],
        );
    }

    #[test]
    fn test_combat3() {
        use self::Team::*;
        let board = b"#######
#E..EG#
#.#G.E#
#E.##E#
#G..#.#
#..E#.#
#######
";
        let game = assert_game(board, 3, 46, 859, 1);
        assert_units(
            &game,
            &[
                (2, 1, Elf, 164),
                (4, 1, Elf, 197),
                (3, 2, Elf, 200),
                (1, 3, Elf, 98),
                (2, 4, Elf, 200),
            ],
        );
        assert_game(board, 4, 33, 948, 0);
    }

    #[test]
    fn test_combat4() {
        use self::Team::*;
        let board = b"#######
#E.G#.#
#.#G..#
#G.#.G#
#G..#.#
#...E.#
#######
";
        let game = assert_game(board, 3, 35, 793, 2);
        assert_units(
            &game,
            &[
                (1, 1, Gnome, 200),
                (3, 1, Gnome, 98),
                (3, 2, Gnome, 200),
                (5, 4, Gnome, 95),
                (4, 5, Gnome, 200),
            ],
        );
        assert_game(board, 15, 37, 94, 0);
    }

    #[test]
    fn test_combat5() {
        use self::Team::*;
        let board = b"#######
#.E...#
#.#..G#
#.###.#
#E#G#G#
#...#G#
#######
";
        let game = assert_game(board, 3, 54, 536, 2);
        assert_units(
            &game,
            &[
                (3, 2, Gnome, 200),
                (1, 5, Gnome, 98),
                (3, 5, Gnome, 38),
                (5, 5, Gnome, 200),
            ],
        );
        assert_game(board, 12, 39, 166, 0);
    }

    #[test]
    fn test_combat6() {
        use self::Team::*;
        let board = b"#########
#G......#
#.E.#...#
#..##..G#
#...##..#
#...#...#
#.G...G.#
#.....G.#
#########
";
        let game = assert_game(board, 3, 20, 937, 1);
        assert_units(
            &game,
            &[
                (2, 1, Gnome, 137),
                (1, 2, Gnome, 200),
                (3, 2, Gnome, 200),
                (2, 3, Gnome, 200),
                (2, 5, Gnome, 200),
            ],
        );
        assert_game(board, 34, 30, 38, 0);
    }

}
