use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink};
use std::{
    fmt::{self, Display},
    io::{self, Read},
    ops::AddAssign,
};

#[derive(Debug, Clone)]
struct GameInfo {
    num_players: u32,
    last_marble: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Score(u32);

impl Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} points", self.0)
    }
}

impl AddAssign<Marble> for Score {
    fn add_assign(&mut self, other: Marble) {
        self.0 += other.value();
    }
}

#[derive(Debug)]
struct Marble(u32);

impl Marble {
    fn value(&self) -> u32 {
        self.0
    }
}

struct Node {
    value: Marble,
    link: LinkedListLink,
}

impl Node {
    fn into_marble(self) -> Marble {
        self.value
    }

    fn value(&self) -> u32 {
        self.value.value()
    }
}

impl From<Marble> for Box<Node> {
    fn from(marble: Marble) -> Self {
        Box::new(Node {
            value: marble,
            link: LinkedListLink::new(),
        })
    }
}

intrusive_adapter!(Adap = Box<Node>: Node { link: LinkedListLink });

fn parse_game_info<R: Read>(mut r: R) -> io::Result<GameInfo> {
    let mut s = String::new();
    r.read_to_string(&mut s)?;
    let mut iter = s
        .split_whitespace()
        .filter_map(|word| word.parse::<u32>().ok());
    let num_players = iter.next().expect("players");
    let last_marble = iter.next().expect("marble");
    assert!(iter.next().is_none());
    Ok(GameInfo {
        num_players,
        last_marble,
    })
}

fn play_game(info: GameInfo) -> Score {
    let mut players = vec![Score::default(); info.num_players as usize];
    let mut marbles = (0..=info.last_marble).map(Marble);
    let mut circle = LinkedList::new(Adap::new());
    circle.push_front(marbles.next().unwrap().into());
    let mut cursor = circle.cursor_mut();
    for marble in marbles {
        let turn = marble.value();
        if turn % 23 == 0 {
            let player = (turn as usize - 1) % players.len();
            let score = &mut players[player];
            *score += marble;
            for _ in 0..7 {
                cursor.move_prev();
                if cursor.is_null() {
                    cursor.move_prev();
                }
            }
            *score += cursor.remove().unwrap().into_marble();
        } else {
            cursor.move_next();
            if cursor.is_null() {
                cursor.move_next();
            }
            debug_assert!(!cursor.is_null());
            cursor.insert_after(marble.into());
            cursor.move_next();
            debug_assert!(cursor.get().unwrap().value() == turn);
        }
    }
    players.into_iter().max().unwrap()
}

fn main() {
    let info = parse_game_info(io::stdin()).unwrap();
    println!("part 1: {}", play_game(info.clone()));
    let info = GameInfo {
        num_players: info.num_players,
        last_marble: 100 * info.last_marble,
    };
    println!("part 2: {}", play_game(info));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1() {
        assert_eq!(
            play_game(GameInfo {
                num_players: 9,
                last_marble: 25,
            }),
            Score(32)
        );
        assert_eq!(
            play_game(GameInfo {
                num_players: 10,
                last_marble: 1618,
            }),
            Score(8317)
        );
        assert_eq!(
            play_game(GameInfo {
                num_players: 13,
                last_marble: 7999,
            }),
            Score(146373)
        );
        assert_eq!(
            play_game(GameInfo {
                num_players: 17,
                last_marble: 1104
            }),
            Score(2764)
        );
        assert_eq!(
            play_game(GameInfo {
                num_players: 21,
                last_marble: 6111,
            }),
            Score(54718)
        );
        assert_eq!(
            play_game(GameInfo {
                num_players: 30,
                last_marble: 5807,
            }),
            Score(37305)
        );
    }
}
