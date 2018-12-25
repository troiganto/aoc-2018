use boolinator::Boolinator;
use std::{
    cmp::Reverse,
    collections::HashSet,
    io::{self, Read},
    num::ParseIntError,
    str::FromStr,
};

fn partition<'a>(s: &'a str, pat: &str) -> (&'a str, Option<&'a str>) {
    let mut words = s.splitn(2, pat);
    (words.next().unwrap(), words.next())
}

#[derive(Debug, Clone)]
#[repr(u8)]
enum Error {
    BadDamage,
    BadElement,
    BadGroup,
    WrongTeam,
    BadParens,
    Missing,
    Excessive,
    UnknownNote,
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Team {
    ImmuneSystem,
    Infection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Element {
    Radiation,
    Bludgeoning,
    Slashing,
    Fire,
    Cold,
}

impl FromStr for Element {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fire" => Ok(Element::Fire),
            "cold" => Ok(Element::Cold),
            "radiation" => Ok(Element::Radiation),
            "bludgeoning" => Ok(Element::Bludgeoning),
            "slashing" => Ok(Element::Slashing),
            _ => Err(Error::BadElement),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Attack {
    power: u32,
    element: Element,
}

impl FromStr for Attack {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_whitespace();
        let power = words.next().ok_or(Error::Missing)?.parse()?;
        let element = words.next().ok_or(Error::Missing)?.parse()?;
        (words.next() == Some("damage")).ok_or(Error::BadDamage)?;
        words.next().is_none().ok_or(Error::Excessive)?;
        Ok(Attack { power, element })
    }
}

impl std::ops::Mul<Attack> for u32 {
    type Output = Attack;

    fn mul(self, rhs: Attack) -> Self::Output {
        Attack {
            power: self * rhs.power,
            element: rhs.element,
        }
    }
}

struct ProtoGroup {
    units: u32,
    hp_per_unit: u32,
    attack_per_unit: Attack,
    weaknesses: HashSet<Element>,
    immunities: HashSet<Element>,
    initiative: u32,
}

impl FromStr for ProtoGroup {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (units, s) = partition(s, " units each with ");
        let (units, s) = (units.parse()?, s.ok_or(Error::Missing)?);
        let (hp_per_unit, s) = partition(s, " hit points ");
        let (hp_per_unit, s) = (hp_per_unit.parse()?, s.ok_or(Error::Missing)?);
        let mut weaknesses = HashSet::new();
        let mut immunities = HashSet::new();
        let s = if s.starts_with('(') {
            let end = s.find(')').ok_or(Error::BadParens)?;
            let notes = &s[1..end];
            for note in notes.split("; ") {
                let (kind, note) = partition(note, " to ");
                let kind: &mut HashSet<Element> = match kind {
                    "weak" => &mut weaknesses,
                    "immune" => &mut immunities,
                    _ => Err(Error::UnknownNote)?,
                };
                let note = note.ok_or(Error::Missing)?;
                for word in note.split(", ") {
                    kind.insert(word.parse()?);
                }
            }
            &s[end + 1..]
        } else {
            s
        };
        let s = partition(s, "with an attack that does ")
            .1
            .ok_or(Error::Missing)?;
        let (attack_per_unit, initiative) = partition(s, " at initiative ");
        let attack_per_unit = attack_per_unit.parse()?;
        let initiative = initiative.ok_or(Error::Missing)?.parse()?;
        Ok(ProtoGroup {
            units,
            hp_per_unit,
            attack_per_unit,
            weaknesses,
            immunities,
            initiative,
        })
    }
}

#[derive(Debug, Clone)]
struct Group {
    team: Team,
    units: u32,
    hp_per_unit: u32,
    attack_per_unit: Attack,
    weaknesses: HashSet<Element>,
    immunities: HashSet<Element>,
    initiative: u32,
}

impl Group {
    fn new(group: ProtoGroup, team: Team) -> Self {
        let ProtoGroup {
            units,
            hp_per_unit,
            attack_per_unit,
            weaknesses,
            immunities,
            initiative,
        } = group;
        Group {
            team,
            units,
            hp_per_unit,
            attack_per_unit,
            weaknesses,
            immunities,
            initiative,
        }
    }

    fn total_attack(&self) -> Attack {
        self.units * self.attack_per_unit.clone()
    }

    fn is_dead(&self) -> bool {
        self.units == 0
    }

    fn take_damage(&mut self, power: u32) -> u32 {
        let old = self.units;
        self.units = self.units.saturating_sub(power / self.hp_per_unit);
        old - self.units
    }

    fn calc_damage(&self, attack: &Attack) -> u32 {
        let &Attack { power, element } = attack;
        if self.immunities.contains(&element) {
            0
        } else if self.weaknesses.contains(&element) {
            power * 2
        } else {
            power
        }
    }
}

fn parse_input(s: &str) -> Result<Vec<Group>, Error> {
    let mut lines = s.lines();
    (lines.next() == Some("Immune System:")).ok_or(Error::WrongTeam)?;
    let mut groups = Vec::with_capacity(24);
    for line in &mut lines {
        if line.is_empty() {
            break;
        }
        groups.push(Group::new(line.parse()?, Team::ImmuneSystem));
    }
    (lines.next() == Some("Infection:")).ok_or(Error::WrongTeam)?;
    for line in &mut lines {
        groups.push(Group::new(line.parse()?, Team::Infection));
    }
    Ok(groups)
}

fn get_target_order(groups: &[Group]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by_key(|&i| {
        let g = &groups[i];
        Reverse((g.total_attack().power, g.initiative))
    });
    order
}

fn get_attack_order(groups: &[Group]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by_key(|&i| Reverse(groups[i].initiative));
    order
}

fn find_winner(groups: &[Group]) -> Option<Team> {
    let mut winner = None;
    for group in groups {
        match winner {
            None => winner = Some(group.team),
            Some(winner) if winner != group.team => return None,
            Some(_) => {},
        }
    }
    winner
}

fn fight(groups: &mut Vec<Group>) -> u32 {
    // 1: Target phase.
    groups.sort_by_key(|g| Reverse((g.total_attack().power, g.initiative)));
    let mut targets = vec![None; groups.len()];
    for i_attacker in get_target_order(&groups) {
        let attacker = &groups[i_attacker];
        let target = groups
            .iter()
            .enumerate()
            .filter(|&(i, _)| !targets.contains(&Some(i)))
            .filter(|&(_, g)| g.team != attacker.team)
            .map(|(i, g)| {
                let damage = g.calc_damage(&attacker.total_attack());
                let key = (damage, g.total_attack().power, g.initiative);
                (i, key)
            })
            .filter(|&(_, (damage, _, _))| damage > 0)
            .max_by_key(|&(_, key)| key)
            .map(|(i, _)| i);
        targets[i_attacker] = target;
    }
    // 2: Attack phase.
    let mut damage_dealt = 0;
    for i_attacker in get_attack_order(&groups) {
        let attack = {
            let attacker = &groups[i_attacker];
            if attacker.is_dead() {
                continue;
            }
            attacker.total_attack()
        };
        let i_defender = match targets[i_attacker] {
            Some(i) => i,
            None => continue,
        };
        let defender = &mut groups[i_defender];
        damage_dealt += defender.take_damage(defender.calc_damage(&attack));
    }
    // 3: Cleanup phase.
    groups.retain(|g| !g.is_dead());
    damage_dealt
}

fn fight_sequence(groups: &[Group], immunoboost: u32) -> (Option<Team>, u32) {
    let mut groups = groups.to_vec();
    for group in &mut groups {
        if let Team::ImmuneSystem = group.team {
            group.attack_per_unit.power += immunoboost;
        }
    }
    let winner = loop {
        let damage_dealt = fight(&mut groups);
        if let Some(winner) = find_winner(&groups) {
            break Some(winner);
        } else if damage_dealt == 0 {
            break None;
        }
    };
    let units: u32 = groups.iter().map(|g| g.units).sum();
    (winner, units)
}

fn find_smallest_boost(groups: &[Group]) -> u32 {
    let (mut min, mut max) = (0, 10);
    // Find a range.
    loop {
        let (winner, _) = fight_sequence(groups, max);
        match winner {
            Some(Team::ImmuneSystem) => break,
            Some(Team::Infection) | None => {
                min = max;
                max *= 2;
            },
        }
    }
    // Narrow the range.
    loop {
        let middle = (min + max) / 2;
        let (winner, _) = fight_sequence(groups, middle);
        if middle == min || middle == max {
            break max;
        }
        match winner {
            Some(Team::ImmuneSystem) => max = middle,
            Some(Team::Infection) | None => min = middle,
        }
    }
}

fn main() {
    let groups = {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        parse_input(&buf).unwrap()
    };
    let (winner, units) = fight_sequence(&groups, 0);
    println!("winner: {:?} with {} units", winner, units);
    let boost = find_smallest_boost(&groups);
    println!("minimum boost: {}", boost);
    let (winner, units) = fight_sequence(&groups, boost);
    println!("winner: {:?} with {} units", winner, units);
}
