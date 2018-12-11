use std::io::{self, BufRead};

type Grid = Box<[[i32; 300]; 300]>;

fn power_levels(serial_number: u32) -> Grid {
    let mut result = Box::new([[0i32; 300]; 300]);
    for x in 1..=300 {
        let rack = x + 10;
        let rack_sq = rack * rack;
        let rack_serial = serial_number * rack;
        for y in 1..=300 {
            let level_base = rack_sq * y + rack_serial;
            let level = ((level_base / 100) % 10) as i32 - 5;
            result[(x - 1) as usize][(y - 1) as usize] = level;
        }
    }
    result
}

fn sum_of_3x3_square(grid: &Grid, topleft: (usize, usize)) -> i32 {
    (0..3)
        .flat_map(|x| (0..3).map(move |y| grid[topleft.0 + x][topleft.1 + y]))
        .sum()
}

fn find_max_3x3_square(grid: &Grid) -> (usize, usize) {
    (0..300 - 2)
        .flat_map(|x| (0..300 - 2).map(move |y| (x, y)))
        .max_by_key(|&pos| sum_of_3x3_square(grid, pos))
        .map(|(x, y)| (x + 1, y + 1))
        .unwrap()
}

fn main() {
    let serial_number = {
        let mut buf = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut buf).unwrap();
        buf.trim().parse::<u32>().unwrap()
    };
    let levels = power_levels(serial_number);
    let (x, y) = find_max_3x3_square(&levels);
    println!("max fuel at: {},{}", x, y);
}
