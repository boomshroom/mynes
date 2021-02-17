use std::num::Wrapping;

use super::{Cpu, Register, StatusFlags};
use crate::decode::{AddressMode, Fix};
use crate::Co;

enum Source {
    Pc,
    Immediate(u16),
}

fn to_le_bytes(val: Wrapping<u16>) -> [Wrapping<u8>; 2] {
    let [low, high] = val.0.to_le_bytes();
    [Wrapping(low), Wrapping(high)]
}

fn from_le_bytes([low, high]: [Wrapping<u8>; 2]) -> Wrapping<u16> {
    Wrapping(u16::from_le_bytes([low.0, high.0]))
}

impl Cpu {
    async fn fetch_add(&mut self, src: Source, offset: u8, fix: Fix, co: &Co<'_>) -> u16 {
        let [low, high] = match src {
            Source::Pc => {
                let low = self.advance(co).await.0;
                let high = self.advance(co).await.0;
                [low, high]
            }
            Source::Immediate(addr) => addr.to_le_bytes(),
        };

        let (effective_low, wrapped) = low.overflowing_add(offset);
        if fix == Fix::Always {
            get!(co, u16::from_le_bytes([effective_low, high]));
        }
        let effective_high = if wrapped {
            if fix == Fix::Conditional {
                get!(co, u16::from_le_bytes([effective_low, high]));
            }
            high.wrapping_add(1)
        } else {
            high
        };
        u16::from_le_bytes([effective_low, effective_high])
    }

    pub(crate) async fn fetch_address(&mut self, mode: AddressMode, co: &Co<'_>) -> Option<u16> {
        match mode {
            AddressMode::Manual => None,
            AddressMode::Implicit => {
                get!(co, self.get_pc());
                None
            }
            AddressMode::Immediate => Some(self.next_pc()),
            AddressMode::ZeroPage => Some(u16::from(self.advance(co).await.0)),
            AddressMode::ZeroPageX => {
                let addr = self.advance(co).await;
                get!(co, u16::from(addr.0));
                Some(u16::from((addr + self.x).0))
            }
            AddressMode::ZeroPageY => {
                let addr = self.advance(co).await;
                get!(co, u16::from(addr.0));
                Some(u16::from((addr + self.y).0))
            }
            AddressMode::Absolute => {
                let low = self.advance(co).await;
                let high = self.advance(co).await;
                Some(u16::from_le_bytes([low.0, high.0]))
            }
            AddressMode::AbsoluteX(fix) => {
                Some(self.fetch_add(Source::Pc, self.x.0, fix, co).await)
            }
            AddressMode::AbsoluteY(fix) => {
                Some(self.fetch_add(Source::Pc, self.y.0, fix, co).await)
            }
            AddressMode::Indirect => {
                let low = self.advance(co).await;
                let high = self.advance(co).await;
                let new_low = get!(co, from_le_bytes([low, high]).0);
                let new_high = get!(co, from_le_bytes([low + Wrapping(1), high]).0);
                Some(u16::from_le_bytes([new_low.0, new_high.0]))
            }
            AddressMode::IndexedIndirect => {
                let table = self.advance(co).await;
                get!(co, u16::from(table.0));
                let entry = table + self.x;
                let low = get!(co, u16::from(entry.0));
                let high = get!(co, u16::from((entry + Wrapping(1_u8)).0));
                Some(u16::from_le_bytes([low.0, high.0]))
            }
            AddressMode::IndirectIndexed(fix) => {
                let addr = self.advance(co).await;
                let low = get!(co, u16::from(addr.0));
                let high = get!(co, u16::from((addr + Wrapping(1_u8)).0));
                Some(
                    self.fetch_add(
                        Source::Immediate(u16::from_le_bytes([low.0, high.0])),
                        self.y.0,
                        fix,
                        co,
                    )
                    .await,
                )
            }
        }
    }

    pub(super) async fn load(&mut self, reg: Register, addr: u16, co: &Co<'_>) {
        let val = get!(co, addr);
        match reg {
            Register::A => self.accum = val,
            Register::X => self.x = val,
            Register::Y => self.y = val,
            Register::S => self.stack = val,
        };
        self.status.z = val.0 == 0;
        self.status.n = (val.0 as i8) < 0;
    }

    pub(super) async fn mem_op(
        &mut self,
        addr: u16,
        op: impl FnOnce(Wrapping<u8>) -> Wrapping<u8>,
        co: &Co<'_>,
    ) -> Wrapping<u8> {
        let val = get!(co, addr);
        set!(co, addr <- val);
        let val = op(val);
        set!(co, addr <- val);
        self.status.z = val.0 == 0;
        self.status.n = (val.0 as i8) < 0;
        val
    }

    pub(super) fn reg_op(&mut self, reg: Register, op: impl FnOnce(Wrapping<u8>) -> Wrapping<u8>) {
        let reg = match reg {
            Register::A => &mut self.accum,
            Register::X => &mut self.x,
            Register::Y => &mut self.y,
            Register::S => &mut self.stack,
        };
        *reg = op(*reg);
        self.status.z = reg.0 == 0;
        self.status.n = (reg.0 as i8) < 0;
    }

    pub(super) fn bin_op(
        &mut self,
        val: Wrapping<u8>,
        op: impl FnOnce(Wrapping<u8>, Wrapping<u8>) -> Wrapping<u8>,
    ) {
        let result = op(self.accum, val);
        self.accum = result;
        self.status.z = result.0 == 0;
        self.status.n = (result.0 as i8) < 0;
    }

    pub(super) async fn shift_op(
        &mut self,
        arg: Option<u16>,
        op: impl FnOnce(Wrapping<u8>, bool) -> (Wrapping<u8>, bool),
        co: &Co<'_>,
    ) -> Wrapping<u8> {
        let (val, carry) = match arg {
            Some(addr) => {
                let val = get!(co, addr);
                set!(co, addr <- val);
                let (val, carry) = op(val, self.status.c);
                set!(co, addr <- val);
                (val, carry)
            }
            None => {
                let (val, carry) = op(self.accum, self.status.c);
                self.accum = val;
                (val, carry)
            }
        };

        self.status.z = val.0 == 0;
        self.status.n = (val.0 as i8) < 0;
        self.status.c = carry;
        val
    }

    pub(super) async fn brk(&mut self, co: &Co<'_>) {
        self.pc += Wrapping(1);
        let [pcl, pch] = to_le_bytes(self.pc);
        set!(co, (self.push()) <- pch);
        set!(co, (self.push()) <- pcl);
        set!(co, (self.push()) <- StatusFlags{ b: true, ..self.status}.load());
        self.set_pcl(get!(co, 0xFFFE));
        self.set_pch(get!(co, 0xFFFF));
        self.status.i = true;
    }

    pub(super) fn transfer(&mut self, src: Register, dst: Register) {
        let val = match src {
            Register::A => self.accum,
            Register::X => self.x,
            Register::Y => self.y,
            Register::S => self.stack,
        };

        match dst {
            Register::A => self.accum = val,
            Register::X => self.x = val,
            Register::Y => self.y = val,
            Register::S => self.stack = val,
        };

        if dst != Register::S {
            self.status.z = val.0 == 0;
            self.status.n = (val.0 as i8) < 0;
        }
    }

    pub(super) async fn jsr(&mut self, co: &Co<'_>) {
        let new_pcl = self.advance(co).await;
        let [pcl, pch] = to_le_bytes(self.pc);
        get!(co, self.peek());
        set!(co, (self.push()) <- pch);
        set!(co, (self.push()) <- pcl);
        self.pc = from_le_bytes([new_pcl, self.advance(co).await]);
    }

    pub(super) async fn rts(&mut self, co: &Co<'_>) {
        get!(co, self.pop());
        let pcl = self.pop();
        self.set_pcl(get!(co, pcl));
        let pch = self.peek();
        self.set_pch(get!(co, pch));
        self.advance(co).await;
    }

    pub(super) async fn rti(&mut self, co: &Co<'_>) {
        get!(co, self.pop());
        self.status = StatusFlags::store(get!(co, self.pop()));
        let pcl = get!(co, self.pop());
        let pch = get!(co, self.peek());
        self.pc = from_le_bytes([pcl, pch]);
    }

    pub(super) async fn pla(&mut self, co: &Co<'_>) {
        get!(co, self.pop());
        self.accum = get!(co, self.peek());

        self.status.z = self.accum.0 == 0;
        self.status.n = (self.accum.0 as i8) < 0;
    }

    pub(super) async fn plp(&mut self, co: &Co<'_>) {
        get!(co, self.pop());
        self.status = StatusFlags::store(get!(co, self.peek()));
    }

    pub(super) async fn branch(&mut self, test: impl FnOnce(&Cpu) -> bool, co: &Co<'_>) -> bool {
        let offset = self.advance(co).await;

        let [old_pcl, old_pch] = to_le_bytes(self.pc);
        if test(&*self) {
            if offset.0 as i8 == -2 {
                return true;
            }

            get!(co, self.get_pc());
            let pcl = old_pcl + offset;
            if (offset.0 as i8) < 0 {
                self.set_pcl(pcl);
                if pcl > old_pcl {
                    get!(co, self.get_pc());
                    self.set_pch(old_pch - Wrapping(1));
                }
            } else {
                self.set_pcl(pcl);
                if pcl < old_pcl {
                    get!(co, self.get_pc());
                    self.set_pch(old_pch + Wrapping(1));
                }
            };
        }

        false
    }

    pub(super) fn bit_test(&mut self, val: u8) {
        self.status.z = val & self.accum.0 == 0;
        self.status.v = val & (1 << 6) != 0;
        self.status.n = (val as i8) < 0;
    }

    pub(super) fn adc(&mut self, val: u8) {
        let (result, carry) = self.accum.0.overflowing_add(val);
        let (result, carry2) = result.overflowing_add(self.status.c as u8);

        let x = (self.accum.0 as i8).is_negative();
        let y = (val as i8).is_negative();
        let r = (result as i8).is_negative();

        self.accum = Wrapping(result);
        self.status.c = carry || carry2;
        self.status.v = (r != x) && (r != y);

        self.status.z = result == 0;
        self.status.n = r;
    }

    pub(super) fn compare(&mut self, reg: Wrapping<u8>, mem: Wrapping<u8>) -> Wrapping<u8> {
        let diff = reg - mem;
        self.status.c = reg >= mem;
        self.status.z = reg == mem;
        self.status.n = (diff.0 as i8).is_negative();
        diff
    }

    pub(super) async fn sra(&mut self, reg: Register, co: &Co<'_>) {
        let low = self.advance(co).await.0;
        let high = self.advance(co).await;

        let (src, mask) = match reg {
            Register::X => (self.y.0, self.x),
            Register::Y => (self.x.0, self.y),
            _ => unreachable!(),
        };

        let (effective_low, wrapped) = low.overflowing_add(src);
        let high_offset = (high + Wrapping(1)) & mask;
        get!(co, u16::from_le_bytes([effective_low, high.0]));
        let effective_high = if wrapped { high_offset } else { high };
        let addr = u16::from_le_bytes([effective_low, effective_high.0]);

        set!(co, addr <- high_offset);
    }

    pub(super) fn xaa(&mut self, imm: Wrapping<u8>) {
        const MAGIC: Wrapping<u8> = Wrapping(0x00);

        let val = (self.accum | MAGIC) & self.x & imm;
        self.accum = val;
        self.status.z = val.0 == 0;
        self.status.n = (val.0 as i8).is_negative();
    }

    pub(super) async fn tas(&mut self, addr: u16, co: &Co<'_>) {
        let val = Wrapping(((addr >> 8) + 1) as u8) & self.accum & self.x;
        set!(co, addr <- val);
        self.stack = val;
    }

    pub(super) fn las(&mut self, arg: Wrapping<u8>) {
        let val = arg & self.stack;
        self.accum = val;
        self.x = val;
        self.stack = val;
        self.status.z = val.0 == 0;
        self.status.n = (val.0 as i8).is_negative();
    }
}
