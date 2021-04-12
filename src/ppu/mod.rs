use std::rc::Rc;
use bounded_integer::bounded_integer;

use crate::memory::Cartridge;

#[cfg(feature = "minifb")]
pub mod backend;
mod loopy;
mod nametable;
mod oam;
mod palette;
pub mod pattern;
mod regs;
pub mod render;

pub use loopy::AddrReg;
use loopy::Time;
pub use nametable::Nametable;
pub use palette::{PaletteRam, PaletteIdx};
use oam::Oam;
use regs::Registers;

bounded_integer!(pub struct VAddr { 0..0x4000 });
bounded_integer!(pub enum NTAddr { 0..4 });
bounded_integer!(#[repr(u16)] pub enum VReg { 0..8 });
bounded_integer!(pub enum PixelCoord { 0..8 });
bounded_integer!(pub enum TileCoord { 0..32 });

pub struct Vram {
    oam: Oam,
    pub palette: PaletteRam,
    pub vram: [Nametable; 2],
    pub registers: Rc<Registers>,

    pub data_bus: u8,
    // pub buffer: Arc<Mutex<[u32, ]>>,
}

impl Default for NTAddr {
    fn default() -> Self { NTAddr::Z }
}

impl NTAddr {
    fn flip_x(self) -> Self {
        match self {
            NTAddr::Z => NTAddr::P1,
            NTAddr::P1 => NTAddr::Z,
            NTAddr::P2 => NTAddr::P3,
            NTAddr::P3 => NTAddr::P2,
        }
    }

    fn flip_y(self) -> Self {
        match self {
            NTAddr::Z => NTAddr::P2,
            NTAddr::P1 => NTAddr::P3,
            NTAddr::P2 => NTAddr::Z,
            NTAddr::P3 => NTAddr::P1,
        }
    }

    fn copy_x(self, src: Self) -> Self {
        match (self, src) {
            (NTAddr::Z | NTAddr::P1, NTAddr::Z | NTAddr::P2) => NTAddr::Z,
            (NTAddr::Z | NTAddr::P1, NTAddr::P1 | NTAddr::P3) => NTAddr::P1,
            (NTAddr::P2 | NTAddr::P3, NTAddr::Z | NTAddr::P2) => NTAddr::P2,
            (NTAddr::P2 | NTAddr::P3, NTAddr::P1 | NTAddr::P3) => NTAddr::P3,
        }
    }

    fn copy_y(self, src: Self) -> Self {
        match (self, src) {
            (NTAddr::Z | NTAddr::P2, NTAddr::Z | NTAddr::P1) => NTAddr::Z,
            (NTAddr::Z | NTAddr::P2, NTAddr::P2 | NTAddr::P3) => NTAddr::P2,
            (NTAddr::P1 | NTAddr::P3, NTAddr::Z | NTAddr::P1) => NTAddr::P1,
            (NTAddr::P1 | NTAddr::P3, NTAddr::P2 | NTAddr::P3) => NTAddr::P3,
        }
    }
}

impl Vram {
    pub fn new() -> Self {
        Vram {
            oam: Oam::new(),
            palette: PaletteRam::default(),
            vram: [Nametable::new(), Nametable::new()],
            registers: Rc::new(Registers::default()),

            data_bus: 0,
        }
    }

    pub fn get_ppu<'c>(&self, addr: VAddr, cart: &Cartridge<'c>) -> u8 {
        match addr.get() {
            0x0000..=0x1FFF => cart.get_ppu(addr),
            0x2000..=0x3EFF => {
                cart.mirror(&self.vram)[usize::from(addr.get() >> 10) % 4].read(addr.get() % 0x400)
            }
            0x3F00..=0x3FFF => self.palette.read((addr.get() % 0x20) as u8),
            _ => unreachable!(),
        }
    }

    pub fn set_ppu<'c>(&mut self, addr: VAddr, val: u8, cart: &mut Cartridge<'c>) {
        match addr.get() {
            0x0000..=0x1FFF => cart.set_ppu(addr, val),
            0x2000..=0x3EFF => {
                let nt = &cart.mirror(&mut self.vram)[usize::from(addr.get() >> 10) % 4];

                nt.write(addr.get() % 0x400, val);
            }
            0x3F00..=0x3FFF => self.palette.write((addr.get() % 0x20) as u8, val),
            _ => unreachable!(),
        }
    }

    pub fn get_cpu(&mut self, addr: VReg) -> (u8, Option<VAddr>) {
        match addr.get() {
            2 => {
                let mut s = self.registers.status.get().into();
                s |= self.data_bus & 0x1F;
                self.registers.set_vblank(false);
                (s, None)
            }
            7 => {
                let addr = self.registers.advance_vaddr();
                if addr >= 0x3F00 {
                    self.data_bus = self.palette.read((addr.get() % 0x20) as u8);
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
            1 => self.registers.mask.set(val.into()),
            5 => {
                self.registers.addr.update(|a| a.write_scroll(val, Time::Delayed));
            },
            6 => {
                self.registers.addr.update(|a| a.write_addr(val));
            },
            7 => {
                let addr = self.registers.advance_vaddr();
                return if addr >= 0x3F00 {
                    self.palette.write((addr.get() % 0x20) as u8, val);
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
