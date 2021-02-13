pub struct NRom<'a> {
    prg_rom: &'a [u8],
    chr_rom: &'a [u8],
    sram: [u8; 0x2000],
}

impl<'a> NRom<'a> {
    pub const fn new(prg_rom: &'a [u8], chr_rom: &'a [u8]) -> Self {
        Self {
            prg_rom,
            chr_rom,
            sram: [0; 0x2000],
        }
    }

    pub fn get(&self, idx: u16) -> u8 {
        match idx {
            0x6000..=0x7fff => self.sram[usize::from(idx) - 0x6000],
            0x8000..=0xffff => self.prg_rom[(usize::from(idx) - 0x8000) % self.prg_rom.len()],
            _ => 0,
        }
    }
    pub fn set(&mut self, idx: u16, val: u8) {
        match idx {
            0x6000..=0x7fff => self.sram[usize::from(idx) - 0x6000] = val,
            _ => (),
        }
    }
}
