#![feature(slice_as_chunks, const_in_array_repeat_expressions)]

use genawaiter::stack::let_gen_using;
use genawaiter::GeneratorState;

macro_rules! get {
    ($co:expr, $addr:expr) => {
        Wrapping($co.yield_(crate::MemoryOp::Read($addr)).await.val)
    };
}

macro_rules! set {
    ($co:expr, $addr:ident <- $val:expr) => {{
        $co.yield_(crate::MemoryOp::Write($addr, $val.0)).await;
    }};
    ($co:expr, ($addr:expr) <- $val:expr) => {{
        $co.yield_(crate::MemoryOp::Write($addr, $val.0)).await;
    }};
}

type Co<'a> = genawaiter::stack::Co<'a, MemoryOp, CycleData>;

mod ppu;
mod audio;
mod cpu;
mod decode;
mod ines;
mod memory;

use ppu::{Vram, VReg};
use audio::Apu;
use cpu::Cpu;
pub use ines::Rom;
use memory::{Cartridge, SysMemory};

pub struct Nes<'a> {
    pub cpu: Cpu,
    bus: MemBus<'a>,
}

struct MemBus<'a> {
    cartridge: Cartridge<'a>,
    memory: SysMemory,
    apu: Apu,
    ppu: Vram,
}

enum MemoryOp {
    Read(u16),
    Write(u16, u8),
}

#[derive(Debug, Copy, Clone)]
struct CycleData {
    val: u8,
    cycles: u64,
}

impl<'a> MemBus<'a> {
    fn get(&mut self, idx: u16) -> u8 {
        match idx {
            0..=0x1fff => self.memory.get(idx),
            0x2000..=0x3FFF => {
                let (byte, addr) = self.ppu.get_cpu(VReg::new_wrapping(idx));
                if let Some(addr) = addr {
                    self.ppu.data_bus = self.ppu.get_ppu(addr, &self.cartridge);
                }
                byte
            }
            0x4015 => self.apu.get_status(),
            0x4020..=0xffff => self.cartridge.get(idx),
            _ => 0,
        }
    }
    fn set(&mut self, idx: u16, val: u8) {
        match idx {
            0..=0x1fff => self.memory.set(idx, val),
            0x2000..=0x3FFF => {
                if let Some(addr) = self.ppu.set_cpu(VReg::new_wrapping(idx), val) {
                    self.ppu.set_ppu(addr, val, &mut self.cartridge);
                }
            }
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu.write(idx, val),
            0x4020..=0xffff => self.cartridge.set(idx, val),
            _ => (),
        }
    }
}

impl<'a> Nes<'a> {
    pub fn new(rom: &'a Rom) -> Self {
        let mut cpu = Cpu::default();

        let mut bus = MemBus {
            cartridge: Cartridge::from_rom(rom),
            memory: SysMemory::new(),
            apu: Apu::new(),
            ppu: Vram::new(),
        };

        cpu.set_pc(u16::from_le_bytes([bus.get(0xfffc), bus.get(0xfffd)]));

        Self { cpu, bus }
    }

    pub fn run(&mut self) -> Result<(), cpu::Error> {
        let Nes { cpu, ref mut bus } = self;

        let_gen_using!(cpu_cycle, |co| cpu.run(co));

        let mut buf = CycleData { val: 0, cycles: 0 };

        loop {
            let op = match cpu_cycle.resume_with(buf) {
                GeneratorState::Yielded(op) => op,
                GeneratorState::Complete(Ok(())) => return Ok(()),
                GeneratorState::Complete(Err(e)) => return Err(e),
            };

            match op {
                MemoryOp::Read(addr) => buf.val = bus.get(addr),
                MemoryOp::Write(addr, val) => bus.set(addr, val),
            };

            if buf.cycles % 2 == 0 {
                bus.apu.clock()
            }
            buf.cycles += 1;
        }
    }

    pub fn set_pc(&mut self, pc: u16) { self.cpu.set_pc(pc); }

    pub fn get_mem(&mut self, addr: u16) -> u8 { self.bus.get(addr) }
}
