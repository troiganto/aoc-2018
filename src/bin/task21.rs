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

fn sum_of_square(grid: &Grid, topleft: (usize, usize), size: usize) -> i32 {
    (0..size)
        .flat_map(|x| (0..size).map(move |y| grid[topleft.0 + x][topleft.1 + y]))
        .sum()
}

fn find_max_square(grid: &Grid, size: usize) -> (usize, usize, i32) {
    (0..301 - size)
        .flat_map(|x| (0..301 - size).map(move |y| (x, y, sum_of_square(grid, (x, y), size))))
        .max_by_key(|&(_, _, fuel)| fuel)
        .map(|(x, y, fuel)| (x + 1, y + 1, fuel))
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
    let (x, y, _) = find_max_square(&levels, 3);
    println!("max fuel at size 3: {},{}", x, y);
    let (x, y, size, _) = (0..301usize)
        .map(|size| {
            let (x, y, fuel) = find_max_square(&levels, size);
            (x, y, size, fuel)
        })
        .max_by_key(|&(_, _, _, fuel)| fuel)
        .unwrap();
    println!("max fuel: {},{},{}", x, y, size);
}
