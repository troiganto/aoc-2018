use std::{
    collections::HashSet,
    io::{self, BufRead},
};

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let freq_steps = stdin
        .lines()
        .map(|line| line.unwrap().trim().parse())
        .collect::<Result<Vec<i32>, _>>()
        .unwrap();
    let mut seen = HashSet::new();
    let mut current = 0i32;
    for step in freq_steps.iter().cycle() {
        if !seen.insert(current) {
            break;
        }
        current += step;
    }
    println!("result: {}", current);
}
