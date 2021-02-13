use crate::ines::{Mapper, Rom};

mod mmc1;
mod nrom;

use mmc1::Mmc1;
use nrom::NRom;

pub const PRG_BANK_SIZE: usize = 0x4000;
pub const CHR_BANK_SIZE: usize = 0x2000;

pub type PRGBank = [u8; PRG_BANK_SIZE];
pub type CHRBank = [u8; CHR_BANK_SIZE];

#[derive(Debug, Clone)]
pub struct SysMemory {
    pub(crate) ram: [u8; 0x800],
    ppu: [u8; 8],
    apu: [u8; 32],
}

pub enum Cartridge<'a> {
    NRom(NRom<'a>),
    Mmc1(Mmc1<'a>),
}

pub struct CPU;
pub struct PPU;

pub trait AddressSpace {}

impl AddressSpace for CPU {}
impl AddressSpace for PPU {}

impl SysMemory {
    pub const fn new() -> Self {
        Self {
            ram: [0; 0x800],
            ppu: [0; 8],
            apu: [0; 32],
        }
    }
}

impl SysMemory {
    pub fn get(&self, idx: u16) -> u8 {
        if idx < 0x1fff {
            self.ram[usize::from(idx) % 0x800]
        } else {
            0
        }
    }
    pub fn set(&mut self, idx: u16, val: u8) {
        if idx < 0x1fff {
            self.ram[usize::from(idx) % 0x800] = val;
        }
    }
}

impl<'a> Cartridge<'a> {
    pub fn get(&self, idx: u16) -> u8 {
        match self {
            Cartridge::NRom(c) => c.get(idx),
            Cartridge::Mmc1(c) => c.get(idx),
        }
    }
    pub fn set(&mut self, idx: u16, val: u8) {
        match self {
            Cartridge::NRom(c) => c.set(idx, val),
            Cartridge::Mmc1(c) => c.set(idx, val),
        }
    }
}

impl<'a> Cartridge<'a> {
    pub fn from_rom(rom: &Rom<'a>) -> Self {
        match rom.mapper() {
            Mapper::NROM => Self::NRom(NRom::new(rom.prg, rom.chr)),
            Mapper::MMC1 => Self::Mmc1(Mmc1::new(rom.prg, rom.chr)),
        }
    }
}
