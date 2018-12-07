use std::{
    collections::BTreeMap,
    io::{self, BufRead},
    iter::{FromIterator, FusedIterator},
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Task(u8);

impl Task {
    fn get_duration(&self) -> u32 {
        (self.0 - b'A' + 1) as u32
    }
}

fn tasks_to_string<T: IntoIterator<Item = Task>>(tasks: T) -> String {
    let bytes = tasks.into_iter().map(|task| task.0).collect();
    String::from_utf8(bytes).unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Prerequisite {
    task: Task,
    needed: Task,
}

impl FromStr for Prerequisite {
    type Err = BadLine;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use boolinator::Boolinator;

        let &needed = s.as_bytes().get(5).ok_or(BadLine)?;
        let &task = s.as_bytes().get(36).ok_or(BadLine)?;
        (b'A'..=b'Z').any(|c| c == needed).ok_or(BadLine)?;
        (b'A'..=b'Z').any(|c| c == task).ok_or(BadLine)?;
        Ok(Prerequisite {
            task: Task(task),
            needed: Task(needed),
        })
    }
}

#[derive(Debug)]
struct BadLine;

#[derive(Clone, Default, Debug)]
struct PrerequisiteTable(BTreeMap<Task, Vec<Task>>);

impl PrerequisiteTable {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn next_doable_task(&mut self) -> Option<Task> {
        use boolinator::Boolinator;

        let next = self
            .0
            .iter()
            .find_map(|(task, prerequisites)| prerequisites.is_empty().as_some(task))
            .cloned();
        if let Some(next) = &next {
            self.0.remove(next);
        }
        next
    }

    fn finish_task(&mut self, done: &Task) {
        for needs in self.0.values_mut() {
            needs.retain(|task| task != done);
        }
    }
}

impl FromIterator<Prerequisite> for PrerequisiteTable {
    fn from_iter<I>(prerequisites: I) -> Self
    where
        I: IntoIterator<Item = Prerequisite>,
    {
        let mut result = BTreeMap::<Task, Vec<Task>>::new();
        for Prerequisite { task, needed } in prerequisites {
            result.entry(task).or_default().push(needed.clone());
            result.entry(needed).or_default();
        }
        PrerequisiteTable(result)
    }
}

impl IntoIterator for PrerequisiteTable {
    type Item = Task;
    type IntoIter = TaskIter;

    fn into_iter(self) -> Self::IntoIter {
        TaskIter(self)
    }
}

struct TaskIter(PrerequisiteTable);

impl Iterator for TaskIter {
    type Item = Task;

    fn next(&mut self) -> Option<Self::Item> {
        let task = self.0.next_doable_task();
        if let Some(task) = &task {
            self.0.finish_task(task);
        }
        task
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for TaskIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for TaskIter {}

#[derive(Debug, Default, Clone)]
struct Worker {
    task: Option<Task>,
    time_until_ready: u32,
}

impl Worker {
    fn is_free(&self) -> bool {
        self.task.is_none()
    }

    fn give_task(&mut self, task: Task, extra_time: u32) -> Result<(), IsBusy> {
        match self.task {
            Some(_) => Err(IsBusy),
            None => {
                self.time_until_ready = task.get_duration() + extra_time;
                self.task = Some(task);
                Ok(())
            },
        }
    }

    fn step(&mut self) -> Option<Task> {
        self.time_until_ready = self.time_until_ready.saturating_sub(1);
        match self.time_until_ready {
            0 => self.task.take(),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct IsBusy;

fn simulate_single_worker(prerequisites: PrerequisiteTable) -> String {
    tasks_to_string(prerequisites.into_iter())
}

fn simulate_parallel_workers(
    mut prerequisites: PrerequisiteTable,
    workers: usize,
    extra_time_per_task: u32,
) -> (String, u32) {
    let mut workers = vec![Worker::default(); workers];
    let mut finished = Vec::new();
    for time in 0.. {
        if !prerequisites.is_empty() {
            // There are remaining tasks, hand them out.
            for worker in &mut workers {
                if worker.is_free() {
                    if let Some(task) = prerequisites.next_doable_task() {
                        worker.give_task(task, extra_time_per_task).unwrap()
                    }
                }
            }
        } else {
            // There are no remaining tasks, wait for workers to finish.
            if workers.iter().all(|w| w.is_free()) {
                return (tasks_to_string(finished), time);
            }
        }
        // Have them perform one unit of work.
        for worker in &mut workers {
            if let Some(task) = worker.step() {
                prerequisites.finish_task(&task);
                finished.push(task);
            }
        }
    }
    panic!("overflow");
}

fn main() {
    let stdin = io::stdin();
    let tasks = stdin
        .lock()
        .lines()
        .map(|line| line.unwrap().parse().unwrap())
        .collect::<PrerequisiteTable>();
    println!(
        "single worker order:   {}",
        simulate_single_worker(tasks.clone())
    );
    let (tasks, time) = simulate_parallel_workers(tasks, 5, 60);
    println!("parallel worker order: {}", tasks);
    println!("parallel worker time: {}", time);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_data() -> PrerequisiteTable {
        vec![
            Prerequisite {
                task: Task(b'A'),
                needed: Task(b'C'),
            },
            Prerequisite {
                task: Task(b'F'),
                needed: Task(b'C'),
            },
            Prerequisite {
                task: Task(b'B'),
                needed: Task(b'A'),
            },
            Prerequisite {
                task: Task(b'D'),
                needed: Task(b'A'),
            },
            Prerequisite {
                task: Task(b'E'),
                needed: Task(b'B'),
            },
            Prerequisite {
                task: Task(b'E'),
                needed: Task(b'D'),
            },
            Prerequisite {
                task: Task(b'E'),
                needed: Task(b'F'),
            },
        ]
        .into_iter()
        .collect::<PrerequisiteTable>()
    }

    #[test]
    fn test_sequential() {
        let prerequisites = example_data();
        let tasks = simulate_single_worker(prerequisites);
        assert_eq!(tasks, "CABDFE");
    }

    #[test]
    fn test_parallel() {
        let prerequisites = example_data();
        let (tasks, time) = simulate_parallel_workers(prerequisites, 2, 0);
        assert_eq!(tasks, "CABFDE");
        assert_eq!(time, 15);
    }
}
