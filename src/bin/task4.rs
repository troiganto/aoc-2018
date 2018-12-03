use std::io::BufRead;

fn differ_by_one(left: &[u8], right: &[u8]) -> bool {
    let mut one_diff = false;
    for (&l, &r) in left.iter().zip(right.iter()) {
        if l != r {
            if one_diff {
                return false;
            } else {
                one_diff = true;
            }
        }
    }
    true
}

fn find_similar<I>(lines: I) -> Option<(String, String)>
where
    I: Iterator,
    I::Item: AsRef<str>,
{
    let mut seen = Vec::<String>::with_capacity(250);
    for line in lines {
        let line = line.as_ref().trim();
        for seen in &seen {
            if differ_by_one(line.as_bytes(), seen.as_bytes()) {
                let line = line.to_owned();
                let seen = seen.clone();
                return Some((line, seen));
            }
        }
        seen.push(line.to_owned());
    }
    None
}

fn remove_differences(left: &str, right: &str) -> String {
    use boolinator::Boolinator;
    left.chars()
        .zip(right.chars())
        .filter_map(|(l, r)| (l == r).as_some(l))
        .collect()
}

fn main() {
    let stdin = std::io::stdin();
    let stdin = stdin.lock();
    let lines = stdin.lines().map(|line| line.unwrap());
    let (left, right) = find_similar(lines).unwrap();
    println!("result: {}", remove_differences(&left, &right));
}
