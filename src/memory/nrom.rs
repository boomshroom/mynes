use crate::ines::Mirroring;
use crate::ppu::pattern::{PTIdx, PatternTableRef};
use crate::ppu::{Nametable, VAddr};

pub struct NRom<'a> {
    prg_rom: &'a [u8],
    chr_rom: &'a [u8],
    sram: [u8; 0x2000],
    mirror: Mirroring,
}

impl<'a> NRom<'a> {
    pub const fn new(prg_rom: &'a [u8], chr_rom: &'a [u8], mirror: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            sram: [0; 0x2000],
            mirror,
        }
    }

    pub fn get(&self, idx: u16) -> u8 {
        match idx {
            0x6000..=0x7fff => self.sram[usize::from(idx) - 0x6000],
            0x8000..=0xffff => self.prg_rom[(usize::from(idx) - 0x8000) % self.prg_rom.len()],
            _ => 0,
        }
    }

    pub fn get_pattern_table(&'a self, idx: PTIdx) -> PatternTableRef<'a> {
        let (tables, rest) = self.chr_rom.as_chunks();
        debug_assert_eq!(rest.len(), 0);
        match idx {
            PTIdx::Left => PatternTableRef(&tables[0]),
            PTIdx::Right => PatternTableRef(&tables[1]),
        }
    }

    pub fn get_ppu(&self, idx: VAddr) -> u8 { self.chr_rom[usize::from(idx.get())] }

    pub fn set(&mut self, idx: u16, val: u8) {
        match idx {
            0x6000..=0x7fff => self.sram[usize::from(idx) - 0x6000] = val,
            _ => (),
        }
    }

    pub fn mirror<'nt>(&self, vram: &'nt [Nametable; 2]) -> [&'nt Nametable; 4] {
        match self.mirror {
            Mirroring::Horizontal => [&vram[0], &vram[0], &vram[1], &vram[1]],
            Mirroring::Vertical => [&vram[0], &vram[1], &vram[0], &vram[1]],
            Mirroring::Ignore => [&vram[0], &vram[0], &vram[0], &vram[0]],
        }
    }
}
