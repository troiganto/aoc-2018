use std::io::BufRead;

fn tally_letters(bytes: &[u8]) -> [u8; 26] {
    let mut tally = [0; 26];
    for byte in bytes {
        let i = match byte {
            b'a'..=b'z' => (byte - b'a') as usize,
            _ => panic!("bad letter: {}", byte),
        };
        tally[i] += 1;
    }
    tally
}

fn main() {
    let stdin = std::io::stdin();
    let stdin = stdin.lock();
    let mut num_doublets = 0;
    let mut num_triplets = 0;
    for line in stdin.lines() {
        let line = line.unwrap();
        let tally = tally_letters(line.trim().as_bytes());
        if tally.iter().any(|&x| x == 2) {
            num_doublets += 1;
        }
        if tally.iter().any(|&x| x == 3) {
            num_triplets += 1;
        }
    }
    let checksum = num_doublets * num_triplets;
    println!("checksum: {}", checksum);
}
