use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use crate::memory::PRG_BANK_SIZE;

pub struct Rom<'a> {
    pub header: Header,
    pub prg: &'a [u8],
    pub chr: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    prg_size: u32,
    chr_size: u32,
    flags6: Flags6,
    flags7: Flags7,
    region: Region,
    mapper: Mapper,
    version: Version,
}

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    Ignore,
}

#[derive(Debug, Clone, Copy)]
pub enum Region {
    NTSC,
    PAL,
}

#[derive(Debug, Clone, Copy)]
pub struct Flags6 {
    mirror: Mirroring,
    battery: bool,
    trainer: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Flags7 {
    vs_unisystem: bool,
    play_choice: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Version {
    Archaic,
    INes,
    Nes2_0,
}

#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    NROM = 0,
    MMC1 = 1,
}

impl<'a> Rom<'a> {
    pub fn parse(rom: &'a [u8]) -> Option<Self> {
        let header = Header::parse(rom)?;

        let rom_start = 16 + if header.flags6.trainer { 512 } else { 0 };
        let rom = &rom[rom_start..];
        let (prg, rom) = rom.split_at(header.prg_size as usize);
        let chr = &rom[..header.chr_size as usize];
        Some(Self { header, prg, chr })
    }

    pub fn version(&self) -> Version { self.header.version }

    pub fn is_play_choice(&self) -> bool { self.header.flags7.play_choice }

    pub fn mapper(&self) -> Mapper { self.header.mapper }

    pub fn mirror(&self) -> Mirroring { self.header.flags6.mirror }
}

impl Header {
    pub fn parse(rom: &[u8]) -> Option<Header> {
        if &rom.get(0..4)? != b"NES\x1a" {
            return None;
        }
        let prg_size = rom[4] as u32 * PRG_BANK_SIZE as u32;
        let chr_size = rom[5] as u32 * 0x2000;
        let flags6 = rom[6].into();
        let flags7 = rom[7].into();
        let region = if rom[9] & 1 == 0 {
            Region::NTSC
        } else {
            Region::PAL
        };

        let version = if rom[7] & 0x0C == 0x08 {
            Version::Nes2_0
        } else if rom[7] & 0x0C == 0x00 && &rom[12..16] == &[0; 4] {
            Version::INes
        } else {
            Version::Archaic
        };

        eprintln!(
            "prg: {} [{:#X}] ({} banks), chr: {} [{:#X}] ({} banks)",
            prg_size, prg_size, rom[4], chr_size, chr_size, rom[5]
        );

        let mapper = match version {
            Version::Archaic => unimplemented!("Archaic iNES ROM"),
            Version::INes => Mapper::try_from(rom[6] >> 4 | rom[7] & 0xF0).unwrap(),
            Version::Nes2_0 => todo!("NES 2.0 ROM"),
        };

        Some(Self {
            prg_size,
            chr_size,
            flags6,
            flags7,
            region,
            version,
            mapper,
        })
    }
}

impl From<u8> for Flags6 {
    fn from(bits: u8) -> Self {
        let mirror = if bits & 8 != 0 {
            Mirroring::Ignore
        } else if bits & 1 == 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
        let battery = bits & 2 != 0;
        let trainer = bits & 4 != 0;
        Self {
            mirror,
            battery,
            trainer,
        }
    }
}

impl From<u8> for Flags7 {
    fn from(bits: u8) -> Self {
        let vs_unisystem = bits & 1 != 0;
        let play_choice = bits & 2 != 0;
        Self {
            vs_unisystem,
            play_choice,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnknownMapper(u16);

impl TryFrom<u8> for Mapper {
    type Error = UnknownMapper;
    fn try_from(id: u8) -> Result<Self, UnknownMapper> { Self::try_from(u16::from(id)) }
}

impl TryFrom<u16> for Mapper {
    type Error = UnknownMapper;
    fn try_from(id: u16) -> Result<Self, UnknownMapper> {
        match id {
            0 => Ok(Self::NROM),
            1 => Ok(Self::MMC1),
            n => Err(UnknownMapper(n)),
        }
    }
}

impl Display for UnknownMapper {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Unknown Mapper ID: {:03x}", self.0)
    }
}
