use boolinator::Boolinator;
use std::{
    collections::HashSet,
    fmt,
    io::{self, BufRead},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum Error {
    BadInst,
    BadRegs,
    BadTest,
    ParseOpError(String),
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Op {
    AddR,
    AddI,
    MulR,
    MulI,
    BanR,
    BanI,
    BorR,
    BorI,
    SetR,
    SetI,
    GtIR,
    GtRI,
    GtRR,
    EqIR,
    EqRI,
    EqRR,
}

impl FromStr for Op {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "addr" => Ok(Op::AddR),
            "addi" => Ok(Op::AddI),
            "mulr" => Ok(Op::MulR),
            "muli" => Ok(Op::MulI),
            "banr" => Ok(Op::BanR),
            "bani" => Ok(Op::BanI),
            "borr" => Ok(Op::BorR),
            "bori" => Ok(Op::BorI),
            "setr" => Ok(Op::SetR),
            "seti" => Ok(Op::SetI),
            "gtir" => Ok(Op::GtIR),
            "gtri" => Ok(Op::GtRI),
            "gtrr" => Ok(Op::GtRR),
            "eqir" => Ok(Op::EqIR),
            "eqri" => Ok(Op::EqRI),
            "eqrr" => Ok(Op::EqRR),
            _ => Err(Error::ParseOpError(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Args {
    a: u32,
    b: u32,
    c: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    op: Op,
    args: Args,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { a, b, c } = self.args;
        write!(f, "{:?} {} {} {}", self.op, a, b, c)
    }
}

impl fmt::Binary for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { a, b, c } = self.args;
        write!(f, "{:?} {:b} {:b} {:b}", self.op, a, b, c)
    }
}

impl fmt::Octal for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { a, b, c } = self.args;
        write!(f, "{:?} {:o} {:o} {:o}", self.op, a, b, c)
    }
}

impl fmt::LowerHex for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { a, b, c } = self.args;
        write!(f, "{:?} {:x} {:x} {:x}", self.op, a, b, c)
    }
}

impl fmt::UpperHex for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { a, b, c } = self.args;
        write!(f, "{:?} {:X} {:X} {:X}", self.op, a, b, c)
    }
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let op = parts.next().ok_or(Error::BadInst)?.parse()?;
        let a = parts.next().ok_or(Error::BadInst)?.parse()?;
        let b = parts.next().ok_or(Error::BadInst)?.parse()?;
        let c = parts.next().ok_or(Error::BadInst)?.parse()?;
        (parts.next().is_none()).ok_or(Error::BadInst)?;
        let args = Args { a, b, c };
        Ok(Instruction { op, args })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Registers([usize; 6]);

impl std::ops::Index<u32> for Registers {
    type Output = usize;

    fn index(&self, idx: u32) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl std::ops::IndexMut<u32> for Registers {
    fn index_mut(&mut self, idx: u32) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

impl Registers {
    fn addr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] + self[b];
    }

    fn addi(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] + b as usize;
    }

    fn mulr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] * self[b];
    }

    fn muli(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] * b as usize;
    }

    fn banr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] & self[b];
    }

    fn bani(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] & b as usize;
    }

    fn borr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] | self[b];
    }

    fn bori(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] | b as usize;
    }

    fn setr(&mut self, args: Args) {
        let Args { a, c, .. } = args;
        self[c] = self[a];
    }

    fn seti(&mut self, args: Args) {
        let Args { a, c, .. } = args;
        self[c] = a as usize;
    }

    fn gtir(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (a as usize > self[b]) as usize;
    }

    fn gtri(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] > b as usize) as usize;
    }

    fn gtrr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] > self[b]) as usize;
    }

    fn eqir(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (a as usize == self[b]) as usize;
    }

    fn eqri(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] == b as usize) as usize;
    }

    fn eqrr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] == self[b]) as usize;
    }

    fn handle(&mut self, Instruction { op, args }: &Instruction) {
        match op {
            Op::AddR => self.addr(*args),
            Op::AddI => self.addi(*args),
            Op::MulR => self.mulr(*args),
            Op::MulI => self.muli(*args),
            Op::BanR => self.banr(*args),
            Op::BanI => self.bani(*args),
            Op::BorR => self.borr(*args),
            Op::BorI => self.bori(*args),
            Op::SetR => self.setr(*args),
            Op::SetI => self.seti(*args),
            Op::GtRI => self.gtri(*args),
            Op::GtIR => self.gtir(*args),
            Op::GtRR => self.gtrr(*args),
            Op::EqRI => self.eqri(*args),
            Op::EqIR => self.eqir(*args),
            Op::EqRR => self.eqrr(*args),
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        write!(f, "[{}", self.0[0])?;
        for reg in &self.0[1..] {
            write!(f, ", {}", reg)?;
        }
        f.write_char(']')
    }
}

impl fmt::Binary for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        write!(f, "[{:b}", self.0[0])?;
        for reg in &self.0[1..] {
            write!(f, ", {:b}", reg)?;
        }
        f.write_char(']')
    }
}

impl fmt::Octal for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        write!(f, "[{:o}", self.0[0])?;
        for reg in &self.0[1..] {
            write!(f, ", {:o}", reg)?;
        }
        f.write_char(']')
    }
}

impl fmt::LowerHex for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        write!(f, "[{:x}", self.0[0])?;
        for reg in &self.0[1..] {
            write!(f, ", {:x}", reg)?;
        }
        f.write_char(']')
    }
}

impl fmt::UpperHex for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        write!(f, "[{:X}", self.0[0])?;
        for reg in &self.0[1..] {
            write!(f, ", {:X}", reg)?;
        }
        f.write_char(']')
    }
}

#[derive(Debug, Clone)]
pub enum ProgramLine {
    Instruction(Instruction),
    IpDecl(u32),
}

impl FromStr for ProgramLine {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#ip ") {
            Ok(ProgramLine::IpDecl(s[4..].parse()?))
        } else {
            Ok(ProgramLine::Instruction(s.parse()?))
        }
    }
}

fn find_correct_value(program: &[Instruction], ipreg: u32) -> usize {
    let mut regs = Registers::default();
    while let Some(inst) = program.get(regs[ipreg]) {
        regs.handle(inst);
        if regs[ipreg] == 28 {
            return regs[3];
        }
        regs[ipreg] += 1;
    }
    unreachable!();
}

fn find_cycle(program: &[Instruction], ipreg: u32) -> usize {
    let mut regs = Registers::default();
    regs.0[0] = 0;
    let mut last_value = 0;
    let mut values_seen = HashSet::new();
    while let Some(inst) = program.get(regs[ipreg]) {
        regs.handle(inst);
        if regs[ipreg] == 28 {
            let value = regs[3];
            if !values_seen.insert(value) {
                return last_value;
            } else {
                last_value = value;
            }
        }
        regs[ipreg] += 1;
    }
    unreachable!();
}

fn read_program<R: BufRead>(mut file: R) -> Result<(Option<u32>, Vec<Instruction>), Error> {
    let mut program = Vec::new();
    let mut ipreg = None;
    let mut buf = String::new();
    while file.read_line(&mut buf).unwrap() > 0 {
        match buf.trim_end().parse()? {
            ProgramLine::Instruction(inst) => program.push(inst),
            ProgramLine::IpDecl(i) => ipreg = Some(i),
        }
        buf.clear();
    }
    Ok((ipreg, program))
}

fn main() {
    let (ipreg, program) = read_program(io::stdin().lock()).unwrap();
    let ipreg = ipreg.unwrap();
    println!("{}", find_correct_value(&program, ipreg));
    println!("{}", find_cycle(&program, ipreg));
}
