use std::fmt::{self, Display, Formatter};
use std::num::Wrapping;

use genawaiter::stack::Co;

use crate::decode::{Instruction, Opcode};
use crate::{CycleData, MemoryOp};

mod instr;

#[derive(Debug)]
pub struct Cpu {
    pub pc: Wrapping<u16>,
    pub stack: Wrapping<u8>,
    pub status: StatusFlags,
    pub accum: Wrapping<u8>,
    pub x: Wrapping<u8>,
    pub y: Wrapping<u8>,
}

#[derive(Debug, Copy, Clone)]
pub struct StatusFlags {
    pub n: bool,
    pub v: bool,
    pub b: bool,
    pub d: bool,
    pub i: bool,
    pub z: bool,
    pub c: bool,
}

impl StatusFlags {
    pub const N: u8 = 0x80;
    pub const V: u8 = 0x40;
    pub const B: u8 = 0x10;
    pub const D: u8 = 0x08;
    pub const I: u8 = 0x04;
    pub const Z: u8 = 0x02;
    pub const C: u8 = 0x01;

    pub fn load(&self) -> Wrapping<u8> {
        let mut ret = 0x30;
        if self.n {
            ret |= Self::N;
        }
        if self.v {
            ret |= Self::V;
        }
        if self.b {
            ret |= Self::B;
        }
        if self.d {
            ret |= Self::D;
        }
        if self.i {
            ret |= Self::I;
        }
        if self.z {
            ret |= Self::Z;
        }
        if self.c {
            ret |= Self::C;
        }
        Wrapping(ret)
    }
    pub fn store(Wrapping(flags): Wrapping<u8>) -> Self {
        Self {
            n: flags & Self::N != 0,
            v: flags & Self::V != 0,
            b: flags & Self::B != 0,
            d: flags & Self::D != 0,
            i: flags & Self::I != 0,
            z: flags & Self::Z != 0,
            c: flags & Self::C != 0,
        }
    }
}

impl From<u8> for StatusFlags {
    #[inline]
    fn from(flags: u8) -> Self { Self::store(Wrapping(flags)) }
}

impl From<StatusFlags> for u8 {
    #[inline]
    fn from(flags: StatusFlags) -> u8 { flags.load().0 }
}

impl Default for StatusFlags {
    fn default() -> Self { Self::store(Wrapping(0x34)) }
}

impl Cpu {
    pub fn next_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += Wrapping(1);
        pc.0
    }

    pub fn get_pc(&self) -> u16 { self.pc.0 }

    pub fn peek(&self) -> u16 { self.stack.0 as u16 | 0x0100 }

    pub fn push(&mut self) -> u16 {
        let stack = self.peek();
        self.stack -= Wrapping(1);
        stack
    }
    pub fn pop(&mut self) -> u16 {
        let stack = self.peek();
        self.stack += Wrapping(1);
        stack
    }

    pub fn set_pcl(&mut self, pcl: Wrapping<u8>) {
        self.pc.0 = (self.pc.0 & 0xFF00) | pcl.0 as u16;
    }
    pub fn set_pch(&mut self, pch: Wrapping<u8>) {
        self.pc.0 = (self.pc.0 & 0x00FF) | ((pch.0 as u16) << 8);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            pc: Wrapping(0),
            stack: Wrapping(0xFD),
            status: StatusFlags::default(),
            accum: Wrapping(0),
            x: Wrapping(0),
            y: Wrapping(0),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum Register {
    A,
    X,
    Y,
    S,
}

#[derive(Debug, Clone)]
pub enum Error {
    UnknownInstr(Instruction, Wrapping<u16>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::UnknownInstr(i, addr) => {
                write!(f, "Unknown instruction: {:?} at {:04x}", i, addr)
            }
        }
    }
}

impl Cpu {
    pub fn set_pc(&mut self, pc: u16) { self.pc.0 = pc; }

    async fn advance(&mut self, co: &Co<'_, MemoryOp, CycleData>) -> Wrapping<u8> {
        get!(co, self.next_pc())
    }

    pub(crate) async fn run(&mut self, co: Co<'_, MemoryOp, CycleData>) -> Result<(), Error> {
        loop {
            let old_pc = self.pc;
            let CycleData { val, cycles: _ } = co.yield_(MemoryOp::Read(self.next_pc())).await;
            let instr = Instruction::decode(val);

            /*
            println!(
                "{:04X}: {:?}\t\t[A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}] CYC:{}",
                old_pc,
                instr.op_code,
                self.accum,
                self.x,
                self.y,
                self.status.load().0 & 0xEF,
                self.stack,
                cycles + 6,
            );
            */

            let addr = self.fetch_address(instr.addr_mode, &co).await;

            match instr.op_code {
                Opcode::NOP => (),

                Opcode::JMP => {
                    let addr = addr.unwrap();
                    if addr == old_pc.0 {
                        return Ok(());
                    } /*else if addr < 0x8000 {
                          panic!("Nonsensical jump target: {:04X}", addr);
                      }*/
                    self.pc = Wrapping(addr);
                }
                Opcode::JSR => self.jsr(&co).await,
                Opcode::BRK => self.brk(&co).await,
                Opcode::RTS => {
                    self.rts(&co).await;
                    if self.pc.0 < 2 {
                        return Ok(());
                    }
                }
                Opcode::RTI => self.rti(&co).await,

                Opcode::PHA => set!(co, (self.push()) <- self.accum),
                Opcode::PHP => set!(co, (self.push()) <- self.status.load()),
                Opcode::PLA => self.pla(&co).await,
                Opcode::PLP => self.plp(&co).await,

                Opcode::CLC => self.status.c = false,
                Opcode::CLD => self.status.d = false,
                Opcode::CLI => self.status.i = false,
                Opcode::CLV => self.status.v = false,
                Opcode::SEC => self.status.c = true,
                Opcode::SED => self.status.d = true,
                Opcode::SEI => self.status.i = true,

                Opcode::TAX => self.transfer(Register::A, Register::X),
                Opcode::TAY => self.transfer(Register::A, Register::Y),
                Opcode::TXA => self.transfer(Register::X, Register::A),
                Opcode::TYA => self.transfer(Register::Y, Register::A),
                Opcode::TSX => self.transfer(Register::S, Register::X),
                Opcode::TXS => self.transfer(Register::X, Register::S),

                Opcode::STA => set!(co, (addr.unwrap()) <- self.accum),
                Opcode::STX => set!(co, (addr.unwrap()) <- self.x),
                Opcode::STY => set!(co, (addr.unwrap()) <- self.y),
                Opcode::LDA => self.load(Register::A, addr.unwrap(), &co).await,
                Opcode::LDX => self.load(Register::X, addr.unwrap(), &co).await,
                Opcode::LDY => self.load(Register::Y, addr.unwrap(), &co).await,

                Opcode::INC => {
                    self.mem_op(addr.unwrap(), |x| x + Wrapping(1_u8), &co)
                        .await;
                }
                Opcode::DEC => {
                    self.mem_op(addr.unwrap(), |x| x - Wrapping(1_u8), &co)
                        .await;
                }
                Opcode::INX => self.reg_op(Register::X, |x| x + Wrapping(1_u8)),
                Opcode::INY => self.reg_op(Register::Y, |x| x + Wrapping(1_u8)),
                Opcode::DEX => self.reg_op(Register::X, |x| x - Wrapping(1_u8)),
                Opcode::DEY => self.reg_op(Register::Y, |x| x - Wrapping(1_u8)),

                Opcode::EOR => self.bin_op(get!(co, addr.unwrap()), |x, y| x ^ y),
                Opcode::ORA => self.bin_op(get!(co, addr.unwrap()), |x, y| x | y),
                Opcode::AND => self.bin_op(get!(co, addr.unwrap()), |x, y| x & y),

                Opcode::ADC => self.adc(get!(co, addr.unwrap()).0),
                Opcode::SBC => self.adc(!get!(co, addr.unwrap()).0),
                Opcode::BIT => self.bit_test(get!(co, addr.unwrap()).0),

                Opcode::CMP => {
                    self.compare(self.accum, get!(co, addr.unwrap()));
                }
                Opcode::CPX => {
                    self.compare(self.x, get!(co, addr.unwrap()));
                }
                Opcode::CPY => {
                    self.compare(self.y, get!(co, addr.unwrap()));
                }

                Opcode::BCS => {
                    if self.branch(|cpu| cpu.status.c, &co).await {
                        return Ok(());
                    }
                }
                Opcode::BEQ => {
                    if self.branch(|cpu| cpu.status.z, &co).await {
                        return Ok(());
                    }
                }
                Opcode::BVS => {
                    if self.branch(|cpu| cpu.status.v, &co).await {
                        return Ok(());
                    }
                }
                Opcode::BMI => {
                    if self.branch(|cpu| cpu.status.n, &co).await {
                        return Ok(());
                    }
                }
                Opcode::BCC => {
                    if self.branch(|cpu| !cpu.status.c, &co).await {
                        return Ok(());
                    }
                }
                Opcode::BNE => {
                    if self.branch(|cpu| !cpu.status.z, &co).await {
                        return Ok(());
                    }
                }
                Opcode::BVC => {
                    if self.branch(|cpu| !cpu.status.v, &co).await {
                        return Ok(());
                    }
                }
                Opcode::BPL => {
                    if self.branch(|cpu| !cpu.status.n, &co).await {
                        return Ok(());
                    }
                }

                Opcode::ASL => {
                    self.shift_op(addr, |x, _| (x << 1, (x.0 as i8) < 0), &co)
                        .await;
                }
                Opcode::LSR => {
                    self.shift_op(addr, |x, _| (x >> 1, x.0 & 1 != 0), &co)
                        .await;
                }
                Opcode::ROL => {
                    self.shift_op(
                        addr,
                        |x, c| (x << 1 | Wrapping(c as u8), (x.0 as i8) < 0),
                        &co,
                    )
                    .await;
                }
                Opcode::ROR => {
                    self.shift_op(
                        addr,
                        |x, c| (x >> 1 | Wrapping(c as u8) << 7, x.0 & 1 != 0),
                        &co,
                    )
                    .await;
                }

                Opcode::NOPConsume => {
                    get!(co, addr.unwrap());
                }
                Opcode::LAX => {
                    self.load(Register::A, addr.unwrap(), &co).await;
                    self.transfer(Register::A, Register::X);
                }
                Opcode::SAX => set!(co, (addr.unwrap()) <- self.accum & self.x),
                Opcode::DCP => {
                    let val = self
                        .mem_op(addr.unwrap(), |x| x - Wrapping(1_u8), &co)
                        .await;
                    self.compare(self.accum, val);
                }
                Opcode::ISB => {
                    let val = self
                        .mem_op(addr.unwrap(), |x| x + Wrapping(1_u8), &co)
                        .await;
                    self.adc(!val.0);
                }
                Opcode::ANC => {
                    self.bin_op(get!(co, addr.unwrap()), |x, y| x & y);
                    self.status.c = self.status.n;
                }
                Opcode::ALR => {
                    self.bin_op(get!(co, addr.unwrap()), |x, y| x & y);
                    self.shift_op(None, |x, _| (x >> 1, x.0 & 1 != 0), &co)
                        .await;
                }
                Opcode::ARR => {
                    self.bin_op(get!(co, addr.unwrap()), |x, y| x & y);
                    let val = self
                        .shift_op(
                            None,
                            |x, c| (x >> 1 | Wrapping(c as u8) << 7, x.0 & (1 << 7) != 0),
                            &co,
                        )
                        .await;
                    self.status.v = ((val >> 6).0 & 1) != ((val >> 5).0 & 1);
                }
                Opcode::AXS => self.x = self.compare(self.accum & self.x, get!(co, addr.unwrap())),
                Opcode::SLO => {
                    let val = self
                        .shift_op(addr, |x, _| (x << 1, (x.0 as i8) < 0), &co)
                        .await;
                    self.bin_op(val, |x, y| x | y);
                }
                Opcode::SRE => {
                    let val = self
                        .shift_op(addr, |x, _| (x >> 1, x.0 & 1 != 0), &co)
                        .await;
                    self.bin_op(val, |x, y| x ^ y);
                }
                Opcode::RLA => {
                    let val = self
                        .shift_op(
                            addr,
                            |x, c| (x << 1 | Wrapping(c as u8), (x.0 as i8) < 0),
                            &co,
                        )
                        .await;
                    self.bin_op(val, |x, y| x & y);
                }
                Opcode::RRA => {
                    let val = self
                        .shift_op(
                            addr,
                            |x, c| (x >> 1 | Wrapping(c as u8) << 7, x.0 & 1 != 0),
                            &co,
                        )
                        .await;
                    self.adc(val.0);
                }
                Opcode::SXA => self.sra(Register::X, &co).await,
                Opcode::SYA => self.sra(Register::Y, &co).await,
                Opcode::XAA => self.xaa(get!(co, addr.unwrap())),
                Opcode::AHX => {
                    set!(co, (addr.unwrap()) <- Wrapping(((addr.unwrap() >> 8) + 1) as u8) & self.accum & self.x)
                }
                Opcode::TAS => self.tas(addr.unwrap(), &co).await,
                Opcode::LAS => self.las(get!(co, addr.unwrap())),
                _ => return Err(Error::UnknownInstr(instr, self.pc - Wrapping(1))),
            }
        }
    }
}
