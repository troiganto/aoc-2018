use std::{
    io::{self, Read},
    ops::Range,
};

fn annihilible(left: u8, right: u8) -> bool {
    left != right && (left.to_ascii_lowercase() == right || left == right.to_ascii_lowercase())
}

fn find_maximally_reducible_sequence(polymer: &[u8], pos: usize) -> Option<Range<usize>> {
    (0..=pos)
        .rev()
        .zip(pos + 1..polymer.len())
        .take_while(|&(l, r)| annihilible(polymer[l], polymer[r]))
        .last()
        .map(|(l, r)| l..r + 1)
}

fn reduce_polymer(polymer: &mut Vec<u8>) {
    let mut i = 0;
    while i < polymer.len() {
        if let Some(range) = find_maximally_reducible_sequence(polymer, i) {
            polymer.drain(range.clone());
            i = range.start;
        } else {
            i += 1;
        }
    }
}

fn to_reduced(mut polymer: Vec<u8>) -> Vec<u8> {
    reduce_polymer(&mut polymer);
    polymer
}

fn main() {
    let contents = {
        let mut contents = Vec::new();
        let stdin = io::stdin();
        stdin.lock().read_to_end(&mut contents).unwrap();
        while let Some(b'\n') = contents.last() {
            contents.pop();
        }
        contents
    };
    // Task 9.
    println!("simple reduction: {}", to_reduced(contents.clone()).len());
    // Task 10.
    let shortest = (b'a'..=b'z')
        .map(|dropped| {
            let mut contents = contents.clone();
            contents.retain(|c| c.to_ascii_lowercase() != dropped);
            to_reduced(contents).len()
        })
        .min()
        .unwrap();
    println!("optimal reduction: {}", shortest);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_reduce(left: &[u8], right: &[u8]) {
        let mut left = left.to_owned();
        reduce_polymer(&mut left);
        assert_eq!(left, right);
    }

    #[test]
    fn test_simple() {
        assert_reduce(b"aA", b"");
        assert_reduce(b"abBA", b"");
        assert_reduce(b"abAB", b"abAB");
        assert_reduce(b"aabAAB", b"aabAAB");
        assert_reduce(b"dabAcCaCBAcCcaDA", b"dabCBAcaDA");
    }
}
