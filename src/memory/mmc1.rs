use std::borrow::Cow;

use super::{CHRBank, PRGBank};
use crate::ines;
use crate::memory::CHR_BANK_SIZE;
use crate::ppu::{Nametable, VAddr};
use crate::ppu::pattern::{PatternTableRef, PTIdx};

pub struct Mmc1<'a> {
    prg_rom: &'a [PRGBank],
    chr: Cow<'a, [[u8; 0x1000]]>,
    sram: Option<[u8; 0x2000]>,

    prg_banks: [&'a PRGBank; 2],
    chr_banks: [usize; 2],
    settings: Settings,

    shift: u8,
    count: u8,
}

struct Settings {
    mirror: Mirroring,
    prg_mode: PRGMode,
    chr_mode: CHRMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mirror: Mirroring::Lower,
            prg_mode: PRGMode::FixLast,
            chr_mode: CHRMode::Full,
        }
    }
}

impl From<u8> for Settings {
    fn from(shift: u8) -> Self {
        let mirror = match shift % 4 {
            0 => Mirroring::Lower,
            1 => Mirroring::Upper,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        };
        let prg_mode = match (shift >> 2) % 4 {
            0 | 1 => PRGMode::Full,
            2 => PRGMode::FixFirst,
            3 => PRGMode::FixLast,
            _ => unreachable!(),
        };
        let chr_mode = if shift & (1 << 4) != 0 {
            CHRMode::Half
        } else {
            CHRMode::Full
        };
        Settings {
            mirror,
            prg_mode,
            chr_mode,
        }
    }
}

impl<'a> Mmc1<'a> {
    pub fn new(prg_rom: &'a [u8], chr_rom: &'a [u8], mirror: ines::Mirroring) -> Self {
        let (prg_rom, extra) = prg_rom.as_chunks();
        debug_assert_eq!(extra, &[] as &[u8]);
        let (chr_rom, extra) = chr_rom.as_chunks();
        debug_assert_eq!(extra, &[] as &[u8]);
        Self {
            prg_rom,
            chr: Cow::Borrowed(chr_rom),
            sram: Some([0; 0x2000]),

            settings: Settings {
                mirror: mirror.into(),
                ..Settings::default()
            },

            prg_banks: [prg_rom.first().unwrap(), prg_rom.last().unwrap()],
            chr_banks: [0, 1],
            shift: 0,
            count: 0,
        }
    }

    pub fn mirror<'nt>(&self, vram: &'nt [Nametable; 2]) -> [&'nt Nametable; 4] {
        match self.settings.mirror {
            Mirroring::Horizontal => [&vram[0], &vram[0], &vram[1], &vram[1]],
            Mirroring::Vertical => [&vram[0], &vram[1], &vram[0], &vram[1]],
            Mirroring::Lower => [&vram[0], &vram[0], &vram[0], &vram[0]],
            Mirroring::Upper => [&vram[1], &vram[1], &vram[1], &vram[1]],
        }
    }
}

const CONTROL: u16 = 0;
const CHR_1: u16 = 1;
const CHR_2: u16 = 2;
const PRG: u16 = 3;

enum Mirroring {
    Lower = 0,
    Upper = 1,
    Vertical = 2,
    Horizontal = 3,
}

impl From<ines::Mirroring> for Mirroring {
    fn from(m: ines::Mirroring) -> Self {
        match m {
            ines::Mirroring::Horizontal => Mirroring::Horizontal,
            ines::Mirroring::Vertical => Mirroring::Vertical,
            ines::Mirroring::Ignore => Mirroring::Lower,
        }
    }
}

enum PRGMode {
    Full,
    FixFirst,
    FixLast,
}

#[derive(Debug, PartialEq, Eq)]
enum CHRMode {
    Full,
    Half,
}

impl<'a> Mmc1<'a> {
    pub fn get(&self, idx: u16) -> u8 {
        let idx = usize::from(idx);
        match idx {
            0x6000..=0x7fff => self
                .sram
                .as_ref()
                .map(|sram| sram[idx % 0x2000])
                .unwrap_or(0),
            0x8000..=0xbfff => self.prg_banks[0][idx - 0x8000],
            0xc000..=0xffff => self.prg_banks[1][idx - 0xc000],
            _ => 0,
        }
    }

    pub fn set(&mut self, idx: u16, val: u8) {
        match idx {
            0x6000..=0x7fff => {
                self.sram
                    .as_mut()
                    .map(|sram| sram[usize::from(idx) - 0x6000] = val);
            }
            0x8000..=0xffff => {
                if val & (1 << 7) != 0 {
                    self.shift = 0;
                    self.count = 0;
                } else {
                    self.shift &= !(1 << self.count);
                    self.shift |= (val & 1) << self.count;
                    self.count += 1;

                    if self.count >= 5 {
                        let shift = self.shift;
                        let val = usize::from(shift);

                        match (idx >> 13) % 4 {
                            CONTROL => {
                                self.settings = Settings::from(shift);
                                /*match self.settings.prg_mode {
                                    PRGMode::FixFirst => self.prg_banks[0] = self.prg_rom.first().unwrap(),
                                    PRGMode::FixLast => self.prg_banks[1] = self.prg_rom.last().unwrap(),
                                    PRGMode::Full => (),
                                }*/
                            }
                            CHR_1 => match self.settings.chr_mode {
                                CHRMode::Full => self.chr_banks = [val & !1, val | 1],
                                CHRMode::Half => self.chr_banks[0] = val,
                            },
                            CHR_2 => {
                                if self.settings.chr_mode == CHRMode::Half {
                                    self.chr_banks[1] = val;
                                }
                            }
                            PRG => {
                                match (shift & (1 << 4) != 0, &self.sram) {
                                    (true, &None) => self.sram = Some([0; 0x2000]),
                                    (false, &Some(_)) => self.sram = None,
                                    _ => (),
                                }
                                let val = val % 0x0F;
                                match self.settings.prg_mode {
                                    PRGMode::Full => {
                                        self.prg_banks =
                                            [&self.prg_rom[val & !1], &self.prg_rom[val | 1]];
                                    }
                                    PRGMode::FixFirst => {
                                        self.prg_banks =
                                            [self.prg_rom.first().unwrap(), &self.prg_rom[val]];
                                    }
                                    PRGMode::FixLast => {
                                        self.prg_banks =
                                            [&self.prg_rom[val], self.prg_rom.last().unwrap()];
                                    }
                                }
                            }
                            _ => unreachable!(),
                        };

                        self.shift = 0;
                        self.count = 0;
                    }
                }
            }
            _ => (),
        }
    }

    pub fn get_pattern_table(&'a self, idx: PTIdx) -> PatternTableRef<'a> {
        let bank = match idx {
            PTIdx::Left => self.chr_banks[0],
            PTIdx::Right => self.chr_banks[1],
        };
        PatternTableRef(&self.chr[bank])
    }

    pub fn get_ppu(&self, idx: VAddr) -> u8 {
        if let 0x0000..=0x1FFF = idx.get() {
            let bank = usize::from(idx.get() / 0x1000);
            self.chr
                .get(self.chr_banks[bank])
                .map(|b| b[usize::from(idx.get() % 0x1000)])
                .unwrap_or(0)
        } else {
            0
        }
    }
    pub fn set_ppu(&mut self, idx: VAddr, val: u8) {
        use std::iter::repeat;
        if let 0x0000..=0x1FFF = idx.get() {
            let bank = self.chr_banks[usize::from(idx.get() / 0x1000)];
            let chr = self.chr.to_mut();
            chr.extend(repeat([0; 0x1000]).take((bank + 1).saturating_sub(chr.len())));
            chr[bank][usize::from(idx.get() % 0x1000)] = val;
        }
    }
}
