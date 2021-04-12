#![feature(
    slice_as_chunks,
    step_trait,
    step_trait_ext,
    cell_update,
)]

use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use image::{ImageBuffer, Pixel};
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

macro_rules! new_wrapping {
    ($t:ty, $val:expr $(,)*) => {{
        assert_eq!(<$t>::MIN_VALUE, 0);
        <$t>::new($val % (<$t>::MAX_VALUE + 1)).unwrap()
    }};
}

type Co<'a> = genawaiter::stack::Co<'a, MemoryOp, CycleData>;

mod audio;
mod cpu;
mod decode;
mod ines;
mod memory;
pub mod ppu;

use audio::Apu;
use cpu::Cpu;
pub use ines::Rom;
use memory::{Cartridge, SysMemory};
#[cfg(feature = "minifb")]
use ppu::backend::Ppu;

use ppu::render::{FrameBuffer, VOp};
use ppu::{VReg, Vram};

pub struct Nes<'a> {
    pub cpu: Cpu,
    pub bus: MemBus<'a>,
}

pub struct MemBus<'a> {
    pub cartridge: Cartridge<'a>,
    memory: SysMemory,
    apu: Apu,
    pub ppu: Vram,
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
                let (byte, addr) = self.ppu.get_cpu(new_wrapping!(VReg, idx));
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
                if let Some(addr) = self.ppu.set_cpu(new_wrapping!(VReg, idx), val) {
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

        let fb = Arc::new(Mutex::new(ImageBuffer::new(256, 240)));

        let_gen_using!(cpu_cycle, |co| cpu.run(co));
        let_gen_using!(ppu_cycle, |co| FrameBuffer::clock(bus.ppu.registers.clone(), co));

        let mut buf = CycleData { val: 0, cycles: 0 };
        let mut vbuf = 0;

        #[cfg(feature = "minifb")]
        let running = Ppu::open(fb.clone());
        #[cfg(not(feature = "minifb"))]
        let running = AtomicBool::new(true);

        let mut temp_fb = None;

        while running.load(Ordering::Relaxed) {
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

            for _ in 0..3 {
                let (cmd, draw) = match ppu_cycle.resume_with(vbuf) {
                    GeneratorState::Yielded(cmd) => cmd,
                    GeneratorState::Complete(never) => never,
                };
                match cmd {
                    VOp::Fetch(addr) => vbuf = bus.ppu.get_ppu(addr, &bus.cartridge),
                    VOp::Nop => (),
                    VOp::Nmi => (), //todo!(),
                };
                if let Some(draw) = draw {
                    let fb = temp_fb.get_or_insert_with(|| fb.lock().unwrap());
                    fb[draw.point] = bus.ppu.palette.get_background(draw.tile, draw.palette).as_rgb().to_bgra()
                } else {
                    temp_fb.take();
                }
            }
            buf.cycles += 1;
        }
        Ok(())
    }

    pub fn set_pc(&mut self, pc: u16) { self.cpu.set_pc(pc); }

    pub fn get_mem(&mut self, addr: u16) -> u8 { self.bus.get(addr) }
}
