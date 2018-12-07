use std::{
    collections::BTreeMap,
    io::{self, BufRead},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Prerequisite {
    task: u8,
    needed: u8,
}

impl FromStr for Prerequisite {
    type Err = BadLine;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use boolinator::Boolinator;

        let &needed = s.as_bytes().get(5).ok_or(BadLine)?;
        let &task = s.as_bytes().get(36).ok_or(BadLine)?;
        (b'A'..=b'Z').any(|c| c == needed).ok_or(BadLine)?;
        (b'A'..=b'Z').any(|c| c == task).ok_or(BadLine)?;
        Ok(Prerequisite { task, needed })
    }
}

#[derive(Debug)]
struct BadLine;

fn collect_prerequisites<I>(prerequisites: I) -> BTreeMap<u8, Vec<u8>>
where
    I: IntoIterator<Item = Prerequisite>,
{
    let mut result = BTreeMap::<u8, Vec<u8>>::new();
    for Prerequisite { task, needed } in prerequisites {
        result.entry(task).or_default().push(needed);
        result.entry(needed).or_default();
    }
    result
}

fn sort_tasks_by_prerequisites<I>(prerequisites: I) -> Vec<u8>
where
    I: IntoIterator<Item = Prerequisite>,
{
    use boolinator::Boolinator;

    let mut prerequisites = collect_prerequisites(prerequisites);
    let mut result = Vec::with_capacity(prerequisites.len());
    while !prerequisites.is_empty() {
        let &task = prerequisites
            .iter()
            .find_map(|(task, prerequisites)| prerequisites.is_empty().as_some(task))
            .unwrap();
        prerequisites.remove(&task);
        result.push(task);
        for needs in prerequisites.values_mut() {
            needs.retain(|&n| n != task);
        }
    }
    result
}

fn main() {
    let stdin = io::stdin();
    let tasks = sort_tasks_by_prerequisites(
        stdin
            .lock()
            .lines()
            .map(|line| line.unwrap().parse().unwrap()),
    );
    println!("correct order: {}", String::from_utf8(tasks).unwrap());
}
