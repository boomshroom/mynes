use std::sync::{Arc, Mutex};

use bounded_integer::bounded_integer;

use crate::memory::Cartridge;

mod loopy;
mod oam;
mod palette;
pub mod pattern;
mod regs;
mod nametable;
pub mod backend;

use oam::Oam;
use regs::Registers;
pub use nametable::Nametable;

bounded_integer!(pub struct VAddr { 0..0x4000 });
bounded_integer!(pub enum NTAddr { 0..4 });
bounded_integer!(#[repr(u16)] pub enum VReg { 0..8 });
bounded_integer!(pub enum PixelCoord { 0..8 });
bounded_integer!(pub enum TileCoord { 0..32 });

pub struct Vram {
    oam: Oam,
    palette: [u8; 64],
    pub vram: [Nametable; 2],
    registers: Registers,

    pub data_bus: u8,
    // pub buffer: Arc<Mutex<[u32, ]>>,
}

impl Default for NTAddr {
    fn default() -> Self { NTAddr::Z }
}

impl Vram {
    pub fn new() -> Self {
        Vram {
            oam: Oam::new(),
            palette: [0; 64],
            vram: [Nametable::new(), Nametable::new()],
            registers: Registers::default(),

            data_bus: 0,
        }
    }

    pub fn get_ppu<'a>(&self, addr: VAddr, cart: &Cartridge<'a>) -> u8 {
        match addr.get() {
            0x0000..=0x1FFF => cart.get_ppu(addr),
            0x2000..=0x3EFF => cart.mirror(&self.vram)[usize::from(addr.get() >> 10) % 4].read(addr.get() % 0x400),
            0x3F00..=0x3FFF => self.palette[usize::from(addr.get() % 0x20)],
            _ => unreachable!(),
        }
    }

    pub fn set_ppu<'a>(&mut self, addr: VAddr, val: u8, cart: &mut Cartridge<'a>) {
        match addr.get() {
            0x0000..=0x1FFF => cart.set_ppu(addr, val),
            0x2000..=0x3EFF => {
                let nt = &cart.mirror(&mut self.vram)[usize::from(addr.get() >> 10) % 4];

                nt.write(addr.get() % 0x400, val);
            }
            0x3F00..=0x3FFF => self.palette[usize::from(addr.get() % 0x20)] = val,
            _ => unreachable!(),
        }
    }

    pub fn get_cpu(&mut self, addr: VReg) -> (u8, Option<VAddr>) {
        match addr.get() {
            2 => {
                let mut s = self.registers.status.into();
                s |= self.data_bus & 0x1F;
                self.registers.clear_vblank();
                (s, None)
            }
            7 => {
                let addr = self.registers.advance_vaddr();
                if addr >= 0x3F00 {
                    self.data_bus = self.palette[usize::from(addr.get() % 0x20)];
                    (self.data_bus, None)
                } else {
                    (self.data_bus, Some(addr))
                }
            }
            _ => (0, None),
        }
    }

    pub fn set_cpu(&mut self, addr: VReg, val: u8) -> Option<VAddr> {
        match addr.get() {
            0 => self.registers.set_control(val),
            1 => self.registers.mask = val.into(),
            5 => self.registers.addr.write_scroll(val),
            6 => self.registers.addr.write_addr(val),
            7 => {
                let addr = self.registers.advance_vaddr();
                return if addr >= 0x3F00 {
                    self.palette[usize::from(addr.get() % 0x20)] = val;
                    None
                } else {
                    Some(addr)
                };
            }
            _ => (),
        };
        None
    }
}

pub struct Point<T> {
    x: T,
    y: T,
}

/*
pub struct BGTile {
    tile: TileData,
    palettes: [TileColor; 4],
}

pub async fn next_tile(addr: &AddrReg) -> BGTile {
    let nt = addr.get_nametable();
    let tile = addr.get_tile();
    let id = read_nametable(nt, tile).await;

    let attr = read_nt_attr(nt, tile).await;
}
*/
