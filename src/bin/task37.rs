use boolinator::Boolinator;
use std::{
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
    a: u8,
    b: u8,
    c: u8,
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

impl std::ops::Index<u8> for Registers {
    type Output = usize;

    fn index(&self, idx: u8) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl std::ops::IndexMut<u8> for Registers {
    fn index_mut(&mut self, idx: u8) -> &mut Self::Output {
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

#[derive(Debug, Clone)]
pub enum ProgramLine {
    Instruction(Instruction),
    IpDecl(u8),
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

fn execute(program: &[Instruction]) -> Registers {
    let mut regs = Registers::default();
    program.iter().for_each(|inst| regs.handle(inst));
    regs
}

fn execute_complex(
    program: &[Instruction],
    ipreg: u8,
    regs: impl Into<Option<Registers>>,
) -> Registers {
    let mut regs = regs.into().unwrap_or_default();
    while let Some(inst) = program.get(regs[ipreg]) {
        regs.handle(inst);
        regs[ipreg] += 1;
    }
    regs
}

fn read_program<R: BufRead>(mut file: R) -> Result<(Option<u8>, Vec<Instruction>), Error> {
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


/// High-level disassembly of my personal program.
fn manually_disassembled(part2: bool) -> u64 {
    let size = 887 + if part2 { 10550400 } else { 0 };
    let mut result = 0;
    for y in 1..=size {
        for x in 1..=size {
            if y * x == size {
                result += y;
            }
        }
    }
    result
}

/// The actual algorithm implemented by the program.
fn sum_of_divisors(num: u64) -> u64 {
    (1..=num).filter(|d| num % d == 0).sum()
}

fn main() {
    let (ipreg, program) = read_program(io::stdin().lock()).unwrap();
    let regs = if let Some(ipreg) = ipreg {
        execute_complex(&program, ipreg, Registers([0, 0, 0, 0, 0, 0]))
    } else {
        execute(&program)
    };
    println!("registers: {:?}", regs.0);
    println!("result of disassembly: {}", manually_disassembled(false));
    println!("result of own function: {}", sum_of_divisors(887));
    println!("part 2 solution: {}", sum_of_divisors(10551287));
}
