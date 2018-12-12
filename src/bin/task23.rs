use std::{
    collections::HashMap,
    fmt::{self, Display},
    io::{self, BufRead},
    iter::FromIterator,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Pot {
    Empty,
    Filled,
}

#[derive(Debug)]
struct BadChar;

impl Pot {
    fn is_filled(&self) -> bool {
        *self == Pot::Filled
    }

    fn as_char(&self) -> char {
        match self {
            Pot::Empty => '.',
            Pot::Filled => '#',
        }
    }

    fn from_byte(b: u8) -> Result<Pot, BadChar> {
        match b {
            b'#' => Ok(Pot::Filled),
            b'.' => Ok(Pot::Empty),
            _ => Err(BadChar),
        }
    }
}

#[derive(Debug, Clone)]
struct Generation {
    pots: Vec<Pot>,
    backup: Vec<Pot>,
    offset: usize,
}

impl Generation {
    fn progress(&mut self, rules: &RulesDict) {
        self.backup.clear();
        self.backup.push(Pot::Empty);
        self.backup.push(Pot::Empty);
        self.backup.push(Pot::Empty);
        self.backup.push(Pot::Empty);
        self.backup.extend(self.pots.windows(5).map(|p| rules[p]));
        self.backup.push(Pot::Empty);
        self.backup.push(Pot::Empty);
        self.backup.push(Pot::Empty);
        self.backup.push(Pot::Empty);
        std::mem::swap(&mut self.pots, &mut self.backup);
        self.offset += 2;
    }

    fn checksum(&self) -> i64 {
        let mut sum = 0i64;
        for (i, p) in self.pots.iter().enumerate() {
            if p.is_filled() {
                sum += i as i64 - self.offset as i64;
            }
        }
        sum
    }
}

impl FromIterator<Pot> for Generation {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Pot>,
    {
        let mut pots = Vec::with_capacity(400);
        let backup = Vec::with_capacity(pots.capacity());
        pots.push(Pot::Empty);
        pots.push(Pot::Empty);
        pots.push(Pot::Empty);
        pots.push(Pot::Empty);
        pots.extend(iter);
        pots.push(Pot::Empty);
        pots.push(Pot::Empty);
        pots.push(Pot::Empty);
        pots.push(Pot::Empty);
        Generation {
            pots,
            backup,
            offset: 4,
        }
    }
}

impl Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        self.pots
            .iter()
            .map(|pot| f.write_char(pot.as_char()))
            .collect()
    }
}

#[derive(Debug, Clone)]
struct Rule {
    pre: [Pot; 5],
    post: Pot,
}

type RulesDict = HashMap<[Pot; 5], Pot>;

impl FromIterator<Rule> for RulesDict {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Rule>,
    {
        iter.into_iter()
            .map(|Rule { pre, post }| (pre, post))
            .collect()
    }
}

#[derive(Debug)]
struct BadRule;

impl FromStr for Rule {
    type Err = BadRule;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use boolinator::Boolinator;

        let mut parts = s.splitn(2, " => ");
        let pre = parts.next().ok_or(BadRule)?.as_bytes();
        let post = parts.next().ok_or(BadRule)?.as_bytes();
        parts.next().is_none().ok_or(BadRule)?;
        (pre.len() == 5).ok_or(BadRule)?;
        (post.len() == 1).ok_or(BadRule)?;

        let pre = [
            Pot::from_byte(pre[0]).map_err(|_| BadRule)?,
            Pot::from_byte(pre[1]).map_err(|_| BadRule)?,
            Pot::from_byte(pre[2]).map_err(|_| BadRule)?,
            Pot::from_byte(pre[3]).map_err(|_| BadRule)?,
            Pot::from_byte(pre[4]).map_err(|_| BadRule)?,
        ];
        let post = Pot::from_byte(post[0]).map_err(|_| BadRule)?;
        Ok(Rule { pre, post })
    }
}

fn read_generation(s: &str) -> Result<Generation, BadChar> {
    let prefix = "initial state: ";
    assert!(s.starts_with(prefix));
    let (_, s) = s.split_at(prefix.len());
    s.bytes().map(Pot::from_byte).collect()
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock().lines();
    let mut generation = read_generation(stdin.next().unwrap().unwrap().trim()).unwrap();
    assert_eq!(stdin.next().unwrap().unwrap(), "");
    let rules = stdin
        .map(|line| line.unwrap().trim().parse::<Rule>())
        .collect::<Result<RulesDict, _>>()
        .unwrap();
    let mut diff = 0;
    let mut old;
    let mut checksum = generation.checksum();
    for i in 0..1000 {
        old = checksum;
        generation.progress(&rules);
        checksum = generation.checksum();
        if i + 1 == 20 {
            // Part 1.
            println!("{}: {}", i + 1, checksum);
        }
        if i > 100 {
            // Assert we've reached the asymptotic point by i=100.
            assert_eq!(checksum - old, diff);
        } else {
            diff = checksum - old;
        }
    }
    // Assume we've reached the asymptotic point here. From here on, the
    // checksum increases constantly by the same amount.
    let almost_infinity = 50_000_000_000i64;
    let result = checksum + diff * (almost_infinity - 1000);
    println!("{}: {}", almost_infinity, result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case() {
        let rules = [
            "...## => #",
            "..#.. => #",
            ".#... => #",
            ".#.#. => #",
            ".#.## => #",
            ".##.. => #",
            ".#### => #",
            "#.#.# => #",
            "#.### => #",
            "##.#. => #",
            "##.## => #",
            "###.. => #",
            "###.# => #",
            "####. => #",
            "..... => .",
            "....# => .",
            "...#. => .",
            "..#.# => .",
            "..##. => .",
            "..### => .",
            ".#..# => .",
            ".##.# => .",
            ".###. => .",
            "#.... => .",
            "#...# => .",
            "#..#. => .",
            "#..## => .",
            "#.#.. => .",
            "#.##. => .",
            "##... => .",
            "##..# => .",
            "##### => .",
        ]
        .iter()
        .map(|r| r.parse::<Rule>())
        .collect::<Result<RulesDict, _>>()
        .unwrap();
        let mut gen = read_generation("initial state: #..#.#..##......###...###").unwrap();
        for _ in 0..20 {
            gen.progress(&rules);
        }
        assert_eq!(gen.checksum(), 325);
    }

    #[test]
    fn test_simple() {
        let rules = [
            "..... => .",
            "....# => .",
            "...#. => .",
            "..#.. => #",
            ".#... => .",
            "#.... => .",
        ]
        .iter()
        .map(|r| r.parse::<Rule>())
        .collect::<Result<RulesDict, _>>()
        .unwrap();
        let mut gen = read_generation("initial state: #").unwrap();
        for _ in 0..20 {
            gen.progress(&rules);
        }
        assert_eq!(gen.to_string().as_bytes()[gen.offset], b'#');
    }
}
