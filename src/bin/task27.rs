use std::io::{self, Read};

enum UpdateKind {
    PushedOne,
    PushedTwo,
}

struct Scoreboard {
    scores: Vec<u8>,
    elves: (usize, usize),
}

impl Scoreboard {
    fn new(max_num_recipes: usize) -> Self {
        let mut scores = Vec::with_capacity(max_num_recipes);
        scores.push(3);
        scores.push(7);
        let elves = (0, 1);
        Scoreboard { scores, elves }
    }

    fn update(&mut self) -> UpdateKind {
        let recipes = (self.scores[self.elves.0], self.scores[self.elves.1]);
        let combined = recipes.0 + recipes.1;
        debug_assert!(combined <= 18);
        let kind = if combined < 10 {
            self.scores.push(combined);
            UpdateKind::PushedOne
        } else {
            self.scores.push(combined / 10);
            self.scores.push(combined % 10);
            UpdateKind::PushedTwo
        };
        self.elves.0 = (self.elves.0 + recipes.0 as usize + 1) % self.scores.len();
        self.elves.1 = (self.elves.1 + recipes.1 as usize + 1) % self.scores.len();
        kind
    }

    fn update_n_recipes(&mut self, recipes: usize) {
        while self.scores.len() < recipes {
            self.update();
        }
    }

    fn scores(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.scores.get(start..).and_then(|s| s.get(..len))
    }

    fn update_until_sequence(&mut self, needle: &[u8]) -> usize {
        for i in 0..self.scores.len() {
            let scores = match self.scores(i, needle.len()) {
                Some(scores) => scores,
                None => break,
            };
            if scores == needle {
                return i;
            }
        }
        loop {
            let kind = self.update();
            let i = self.scores.len().saturating_sub(needle.len());
            if let UpdateKind::PushedTwo = kind {
                let i = i.saturating_sub(1);
                if self.scores(i, needle.len()) == Some(needle) {
                    return i;
                }
            }
            if self.scores(i, needle.len()) == Some(needle) {
                return i;
            }
        }
    }
}

fn write_scores(scores: &[u8]) -> String {
    let mut buf = String::with_capacity(scores.len());
    for score in scores {
        let c = match score {
            0 => '0',
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            _ => panic!("bad score"),
        };
        buf.push(c);
    }
    buf
}

fn to_digits(s: &str) -> Vec<u8> {
    s.chars().map(|c| c.to_digit(10).unwrap() as u8).collect()
}

fn main() {
    let (stop, needle) = {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        let buf = buf.trim();
        let stop = buf.trim().parse::<usize>().unwrap();
        let needle = to_digits(buf);
        (stop, needle)
    };
    {
        let mut board = Scoreboard::new(stop + 20);
        board.update_n_recipes(stop + 10);
        println!("scores: {}", write_scores(&board.scores[stop..stop + 10]));
    }
    {
        let mut board = Scoreboard::new(1000);
        let i = board.update_until_sequence(&needle);
        println!("pos: {}", i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let mut board = Scoreboard::new(4036);
        board.update_n_recipes(2028);
        assert_eq!(write_scores(&board.scores[5..15]), "0124515891");
        assert_eq!(write_scores(&board.scores[9..19]), "5158916779");
        assert_eq!(write_scores(&board.scores[18..28]), "9251071085");
        assert_eq!(write_scores(&board.scores[2018..2028]), "5941429882");
    }

    #[test]
    fn test_part2() {
        let mut board = Scoreboard::new(4036);
        assert_eq!(board.update_until_sequence(&[5, 1, 5, 8, 9]), 9);
        assert_eq!(board.update_until_sequence(&[0, 1, 2, 4, 5]), 5);
        assert_eq!(board.update_until_sequence(&[9, 2, 5, 1, 0]), 18);
        assert_eq!(board.update_until_sequence(&[5, 9, 4, 1, 4]), 2018);
    }
}
