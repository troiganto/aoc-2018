use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let freq_steps = stdin
        .lines()
        .map(|line| line.unwrap().trim().parse::<i32>().unwrap());
    let sum = freq_steps.sum::<i32>();
    println!("sum: {}", sum);
}
