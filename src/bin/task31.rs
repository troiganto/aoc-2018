use boolinator::Boolinator;
use std::{
    io::{self, BufRead},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Debug, Clone)]
enum Error {
    BadInst,
    BadRegs,
    BadTest,
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Args {
    a: u8,
    b: u8,
    c: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Instruction {
    op: u8,
    args: Args,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Op {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpTable([Op; 16]);

impl OpTable {
    fn is_consistent(&self) -> bool {
        for i in 0..16 {
            for j in i + 1..16 {
                if self[i] == self[j] {
                    return false;
                }
            }
        }
        true
    }
}

impl Default for OpTable {
    fn default() -> Self {
        use self::Op::*;
        OpTable([
            AddR, AddI, MulR, MulI, BanR, BanI, BorR, BorI, SetR, SetI, GtIR, GtRI, GtRR, EqIR,
            EqRI, EqRR,
        ])
    }
}

impl std::ops::Index<u8> for OpTable {
    type Output = Op;

    fn index(&self, idx: u8) -> &Self::Output {
        &self.0[idx as usize]
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Registers([u16; 4]);

impl std::ops::Index<u8> for Registers {
    type Output = u16;

    fn index(&self, idx: u8) -> &u16 {
        &self.0[idx as usize]
    }
}

impl std::ops::IndexMut<u8> for Registers {
    fn index_mut(&mut self, idx: u8) -> &mut u16 {
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
        self[c] = self[a] + b as u16;
    }

    fn mulr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] * self[b];
    }

    fn muli(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] * b as u16;
    }

    fn banr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] & self[b];
    }

    fn bani(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] & b as u16;
    }

    fn borr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] | self[b];
    }

    fn bori(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = self[a] | b as u16;
    }

    fn setr(&mut self, args: Args) {
        let Args { a, c, .. } = args;
        self[c] = self[a];
    }

    fn seti(&mut self, args: Args) {
        let Args { a, c, .. } = args;
        self[c] = a as u16;
    }

    fn gtir(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (a as u16 > self[b]) as u16;
    }

    fn gtri(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] > b as u16) as u16;
    }

    fn gtrr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] > self[b]) as u16;
    }

    fn eqir(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (a as u16 == self[b]) as u16;
    }

    fn eqri(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] == b as u16) as u16;
    }

    fn eqrr(&mut self, args: Args) {
        let Args { a, b, c } = args;
        self[c] = (self[a] == self[b]) as u16;
    }

    fn handle(&mut self, op: Op, args: Args) {
        use self::Op::*;
        match op {
            AddR => self.addr(args),
            AddI => self.addi(args),
            MulR => self.mulr(args),
            MulI => self.muli(args),
            BanR => self.banr(args),
            BanI => self.bani(args),
            BorR => self.borr(args),
            BorI => self.bori(args),
            SetR => self.setr(args),
            SetI => self.seti(args),
            GtRI => self.gtri(args),
            GtIR => self.gtir(args),
            GtRR => self.gtrr(args),
            EqRI => self.eqri(args),
            EqIR => self.eqir(args),
            EqRR => self.eqrr(args),
        }
    }
}

impl FromStr for Registers {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.starts_with("[").ok_or(Error::BadRegs)?;
        s.ends_with("]").ok_or(Error::BadRegs)?;
        let mut parts = s[1..s.len() - 1].split(", ");
        let a = parts.next().ok_or(Error::BadRegs)?.parse()?;
        let b = parts.next().ok_or(Error::BadRegs)?.parse()?;
        let c = parts.next().ok_or(Error::BadRegs)?.parse()?;
        let d = parts.next().ok_or(Error::BadRegs)?.parse()?;
        (parts.next().is_none()).ok_or(Error::BadRegs)?;
        Ok(Registers([a, b, c, d]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cpu {
    ops: OpTable,
    regs: Registers,
}

impl Cpu {
    fn new(ops: OpTable, regs: Registers) -> Self {
        Cpu { ops, regs }
    }

    fn handle(&mut self, inst: Instruction) {
        let Instruction { op, args } = inst;
        let op = self.ops[op];
        self.regs.handle(op, args);
    }
}

#[derive(Debug, Clone)]
struct Testcase {
    before: Registers,
    inst: Instruction,
    after: Registers,
}

impl Testcase {
    fn matches_op(&self, op: Op) -> bool {
        let mut regs = self.before.clone();
        regs.handle(op, self.inst.args);
        regs == self.after
    }

    fn is_table_valid(&self, ops: OpTable) -> bool {
        let mut cpu = Cpu::new(ops, self.before.clone());
        cpu.handle(self.inst);
        cpu.regs == self.after
    }

    fn matching_opcodes(&self) -> impl Iterator<Item = Op> {
        let Testcase {
            before,
            after,
            inst: Instruction { args, .. },
        } = self.clone();
        let table = OpTable::default();
        (0..16u8).map(move |i| table[i]).filter(move |&op| {
            let mut regs = before.clone();
            regs.handle(op, args);
            regs == after
        })
    }

    fn count_matching_opcodes(&self) -> usize {
        self.matching_opcodes().count()
    }
}

impl FromStr for Testcase {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let before = lines.next().ok_or(Error::BadTest)?;
        let inst = lines.next().ok_or(Error::BadTest)?.trim().parse()?;
        let after = lines.next().ok_or(Error::BadTest)?;
        before.starts_with("Before: ").ok_or(Error::BadTest)?;
        let before = before["Before: ".len()..].trim().parse()?;
        after.starts_with("After: ").ok_or(Error::BadTest)?;
        let after = after["After: ".len()..].trim().parse()?;
        Ok(Testcase {
            before,
            inst,
            after,
        })
    }
}

fn read_input<R: BufRead>(mut file: R) -> Result<(Vec<Testcase>, Vec<Instruction>), io::Error> {
    let mut tests = Vec::new();
    let mut program = Vec::new();
    let mut buf = String::new();
    loop {
        let bytes_in_first_lines = file.read_line(&mut buf)?;
        if bytes_in_first_lines < 3 {
            break;
        }
        file.read_line(&mut buf)?;
        file.read_line(&mut buf)?;
        tests.push(buf.parse().expect("could not parse test"));
        let bytes_in_empty_line = file.read_line(&mut buf)?;
        if bytes_in_empty_line != 1 {
            panic!("expected empty line, got {} bytes", bytes_in_empty_line);
        }
        buf.clear();
    }
    assert!(!tests.is_empty());
    if file.read_line(&mut buf)? != 1 {
        panic!("expected empty line");
    }
    buf.clear();
    while file.read_line(&mut buf)? > 0 {
        program.push(buf.trim().parse().expect("could not parse program"));
        buf.clear();
    }
    Ok((tests, program))
}

#[derive(Debug)]
struct OpTableCandidates {
    candidates: [Vec<Op>; 16],
}

impl OpTableCandidates {
    fn new() -> Self {
        let candidates = [
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
            Vec::from(&OpTable::default().0[..]),
        ];
        OpTableCandidates { candidates }
    }

    fn definite_op(&self, opcode: u8) -> Option<Op> {
        let candidates = &self.candidates[opcode as usize];
        if candidates.len() > 1 {
            None
        } else {
            candidates.first().cloned()
        }
    }

    fn filter_by_testcase(&mut self, test: &Testcase) -> Option<Op> {
        let candidates = &mut self.candidates[test.inst.op as usize];
        if candidates.len() > 1 {
            candidates.retain(|&op| test.matches_op(op));
            assert!(!candidates.is_empty(), "filter excluded all ops");
        }
        self.definite_op(test.inst.op)
    }

    fn filter_by_definite_op(&mut self, op: Op) -> Vec<Op> {
        let mut result = Vec::new();
        for candidates in &mut self.candidates {
            if candidates.len() > 1 {
                candidates.retain(|&candidate| candidate != op);
                assert!(!candidates.is_empty(), "definite op excluded all ops");
                if candidates.len() == 1 {
                    result.push(candidates.first().cloned().unwrap());
                }
            }
        }
        result
    }

    fn reduce(&mut self, test: &Testcase) -> bool {
        if let Some(new_found) = self.filter_by_testcase(test) {
            let mut stack = vec![new_found];
            while let Some(found) = stack.pop() {
                stack.extend(self.filter_by_definite_op(found));
            }
        }
        self.candidates.iter().all(|c| c.len() == 1)
    }

    fn into_table(self) -> Result<OpTable, Self> {
        let mut result = OpTable::default().0;
        for (dest, src) in result.iter_mut().zip(self.candidates.iter()) {
            if src.len() != 1 {
                return Err(self);
            } else {
                *dest = src.first().cloned().unwrap();
            }
        }
        Ok(OpTable(result))
    }
}

fn execute(program: &[Instruction], ops: OpTable) -> Registers {
    let mut cpu = Cpu::new(ops, Registers::default());
    for &inst in program {
        cpu.handle(inst);
    }
    cpu.regs
}

fn main() {
    let (tests, program) = read_input(io::stdin().lock()).unwrap();
    let num_super_ambiguous = tests
        .iter()
        .filter(|test| test.count_matching_opcodes() >= 3)
        .count();
    println!("#total: {}", tests.len());
    println!("#ambiguous: {}", num_super_ambiguous);
    let table = || -> OpTable {
        let mut searcher = OpTableCandidates::new();
        for test in &tests {
            if searcher.reduce(test) {
                return searcher.into_table().expect("bad searcher");
            }
        }
        panic!("no solution found");
    }();

    println!(
        "#passing: {}",
        tests
            .iter()
            .filter(|test| test.is_table_valid(table.clone()))
            .count(),
    );
    println!("op table: {:?}", table);
    assert!(table.is_consistent());
    println!("program instructions: {}", program.len());
    println!("program result: {:?}", execute(&program, table));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testcase() {
        let test = Testcase {
            before: Registers([3, 2, 1, 1]),
            after: Registers([3, 2, 2, 1]),
            inst: Instruction {
                op: 9,
                args: Args { a: 2, b: 1, c: 2 },
            },
        };
        assert_eq!(test.count_matching_opcodes(), 3);
    }

    #[test]
    fn test_instructions() {
        let mut regs = Registers::default();
        regs.seti(Args { a: 4, b: 0, c: 0 });
        assert_eq!(regs, Registers([4, 0, 0, 0]));
        regs.setr(Args { a: 0, b: 0, c: 2 });
        assert_eq!(regs, Registers([4, 0, 4, 0]));
        regs.eqrr(Args { a: 0, b: 2, c: 1 });
        assert_eq!(regs, Registers([4, 1, 4, 0]));
        regs.eqri(Args { a: 0, b: 4, c: 3 });
        assert_eq!(regs, Registers([4, 1, 4, 1]));
        regs.eqir(Args { a: 2, b: 3, c: 0 });
        assert_eq!(regs, Registers([0, 1, 4, 1]));
        regs.addr(Args { a: 2, b: 3, c: 0 });
        assert_eq!(regs, Registers([5, 1, 4, 1]));
        regs.addi(Args { a: 0, b: 9, c: 1 });
        assert_eq!(regs, Registers([5, 14, 4, 1]));
        regs.gtrr(Args { a: 2, b: 1, c: 3 });
        assert_eq!(regs, Registers([5, 14, 4, 0]));
        regs.gtri(Args { a: 1, b: 9, c: 0 });
        assert_eq!(regs, Registers([1, 14, 4, 0]));
        regs.gtir(Args { a: 9, b: 2, c: 3 });
        assert_eq!(regs, Registers([1, 14, 4, 1]));
        regs.mulr(Args { a: 2, b: 1, c: 0 });
        assert_eq!(regs, Registers([56, 14, 4, 1]));
        regs.muli(Args { a: 0, b: 0, c: 0 });
        assert_eq!(regs, Registers([0, 14, 4, 1]));
        regs.bani(Args { a: 1, b: 7, c: 0 });
        assert_eq!(regs, Registers([6, 14, 4, 1]));
        regs.banr(Args { a: 1, b: 3, c: 3 });
        assert_eq!(regs, Registers([6, 14, 4, 0]));
        regs.bori(Args { a: 1, b: 1, c: 1 });
        assert_eq!(regs, Registers([6, 15, 4, 0]));
        regs.borr(Args { a: 2, b: 3, c: 0 });
        assert_eq!(regs, Registers([4, 15, 4, 0]));
    }
}
