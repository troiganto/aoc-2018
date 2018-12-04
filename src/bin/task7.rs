use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Time {
    hour: u8,
    minute: u8,
}

impl Time {
    fn as_minutes(self) -> u16 {
        self.hour as u16 * 60 + self.minute as u16
    }

    fn minutes_since(self, other: Time) -> Option<u16> {
        if other < self {
            Some(self.as_minutes() - other.as_minutes())
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Timestamp {
    date: Date,
    time: Time,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct GuardId(u32);

impl Display for GuardId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum EventKind {
    ShiftBegin(GuardId),
    FallsAsleep,
    WakesUp,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Event {
    stamp: Timestamp,
    kind: EventKind,
}

mod parsers {
    use super::{Date, Event, EventKind, GuardId, Time, Timestamp};
    use nom::*;
    use std::str::FromStr;

    named!(date(&str) -> Date,
        do_parse!(
            year: map_res!(digit, u16::from_str) >>
            char!('-') >>
            month: map_res!(digit, u8::from_str) >>
            char!('-') >>
            day: map_res!(digit, u8::from_str) >>
            (Date { year, month, day })
        )
    );

    named!(time(&str) -> Time,
        do_parse!(
            hour: map_res!(digit, u8::from_str) >>
            char!(':') >>
            minute: map_res!(digit, u8::from_str) >>
            (Time { hour, minute })
        )
    );

    named!(timestamp(&str) -> Timestamp,
        do_parse!(
            char!('[') >>
            date: date >>
            char!(' ') >>
            time: time >>
            char!(']') >>
            (Timestamp { date, time })
        )
    );

    named!(guard_id(&str) -> GuardId,
        do_parse!(
            tag!("#") >>
            id: map_res!(digit, u32::from_str) >>
            (GuardId(id))
        )
    );

    named!(event_kind(&str) -> EventKind,
        alt!(
            value!(EventKind::FallsAsleep, tag!("falls asleep"))
            |
            value!(EventKind::WakesUp, tag!("wakes up"))
            |
            map!(
                delimited!(tag!("Guard "), guard_id, tag!(" begins shift")),
                EventKind::ShiftBegin
            )
        )
    );

    named!(event(&str) -> Event,
        do_parse!(
            stamp: timestamp >>
            char!(' ') >>
            kind: event_kind >>
            (Event {stamp, kind})
        )
    );

    #[derive(Debug, Clone)]
    pub struct ParseEventError;

    impl FromStr for Event {
        type Err = ParseEventError;

        fn from_str(input: &str) -> Result<Self, Self::Err> {
            match event(input) {
                Ok(("", event)) => Ok(event),
                _ => Err(ParseEventError),
            }
        }
    }
}

#[derive(Clone, Debug)]
enum ScheduleError {
    ShiftChangeWhileSleep,
    MissingGuard,
    DoubleSleep,
    MissingSleep,
    SleepAtMidnight,
}

type Schedule = HashMap<GuardId, Vec<(Time, Time)>>;

fn build_sleep_schedules(events: &[Event]) -> Result<Schedule, ScheduleError> {
    use boolinator::Boolinator;

    let mut schedule = Schedule::new();
    let mut current_guard = None;
    let mut sleep_begin = None;
    for event in events {
        match event.kind {
            EventKind::ShiftBegin(id) => {
                sleep_begin
                    .is_none()
                    .ok_or(ScheduleError::ShiftChangeWhileSleep)?;
                current_guard = Some(id);
            },
            EventKind::FallsAsleep => {
                current_guard.is_some().ok_or(ScheduleError::MissingGuard)?;
                sleep_begin.is_none().ok_or(ScheduleError::DoubleSleep)?;
                sleep_begin = Some(event.stamp);
            },
            EventKind::WakesUp => {
                let id = current_guard.ok_or(ScheduleError::MissingGuard)?;
                let sleep_begin = sleep_begin.take().ok_or(ScheduleError::MissingSleep)?;
                (sleep_begin.date == event.stamp.date).ok_or(ScheduleError::SleepAtMidnight)?;
                schedule
                    .entry(id)
                    .or_default()
                    .push((sleep_begin.time, event.stamp.time))
            },
        }
    }
    Ok(schedule)
}

fn find_sleepiest_minute(sleep_intervals: &[(Time, Time)]) -> u8 {
    let mut tally = [0; 60];
    let common_hour = match sleep_intervals.first() {
        Some(&(Time { hour, .. }, Time { .. })) => hour,
        None => return 0,
    };
    for &(begin, end) in sleep_intervals {
        assert!(begin.hour == common_hour);
        assert!(end.hour == common_hour);
        for minute in begin.minute..end.minute {
            tally[minute as usize] += 1;
        }
    }
    tally
        .iter()
        .cloned()
        .enumerate()
        .max_by_key(|&(_, value)| value)
        .map(|(i, _)| i as u8)
        .unwrap()
}

fn main() {
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    let mut events = stdin
        .lock()
        .lines()
        .map(|line| line.unwrap().trim().parse())
        .collect::<Result<Vec<Event>, _>>()
        .unwrap();
    events.sort_by_key(|event| event.stamp);
    let schedule = build_sleep_schedules(&events).unwrap();
    let sleepiest_guard = schedule
        .iter()
        .max_by_key(|(_, times)| {
            times
                .iter()
                .cloned()
                .map(|(begin, end)| end.minutes_since(begin).unwrap() as u32)
                .sum::<u32>()
        })
        .map(|(id, _)| id)
        .unwrap();
    let sleepiest_minute = find_sleepiest_minute(&schedule[sleepiest_guard]);
    println!("answer: {}", sleepiest_guard.0 * sleepiest_minute as u32);
}
