use std::cell::Cell;
use super::loopy::{AddrReg, Time};
use super::pattern::PTIdx;
use super::{NTAddr, VAddr, TileCoord, PixelCoord};

#[derive(Default, Debug, Clone)]
pub struct Registers {
    pub control: Cell<Control>,
    pub mask: Cell<Mask>,
    pub status: Cell<Status>,
    oam_addr: u8,
    oam_data: u8,
    pub addr: Cell<AddrReg>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Control {
    base_nt: NTAddr,
    vram_inc: u16,
    sprite_table: PTIdx,
    pub bg_table: PTIdx,
    sprite_height: u8,
    interrupt: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Status {
    overflow: bool,
    zero_hit: bool,
    vblank: bool,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Mask {
    color: Color,
    background_left: Show,
    sprites_left: Show,
    pub background: Show,
    sprites: Show,
    red: Emphasis,
    green: Emphasis,
    blue: Emphasis,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Color {
    Normal,
    Greyscale,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Show {
    Show,
    Hide,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Emphasis {
    On,
    Off,
}

impl Registers {
    pub fn set_vblank(&self, val: bool) { self.status.update(|s| Status{ vblank: val, ..s }); }

    pub fn advance_vaddr(&self) -> VAddr {
        let mut reg = self.addr.get();
        let addr = reg.advance(self.control.get().vram_inc);
        self.addr.set(reg);
        addr
    }

    pub fn set_control(&self, val: u8) {
        let reg = val.into();
        self.control.set(reg);
        self.addr.update(|mut a| { a.set_nametable(reg.base_nt, Time::Delayed); a });
    }

    pub fn enabled(&self) -> bool {
        let mask = self.mask.get();
        mask.background == Show::Show || mask.sprites == Show::Show
    }

    pub fn interrupt_enabled(&self) -> bool {
        self.control.get().interrupt
    }

    pub fn increment_scrollx(&self) {
        if self.enabled() {
            let mut addr = self.addr.get();
            let coarse_x = addr.get_coarse_x();
            if coarse_x == 31 {
                addr.set_coarse_x(TileCoord::Z, Time::Immediate);
                let nt = addr.get_nametable();
                addr.set_nametable(nt.flip_x(), Time::Immediate);
            } else {
                addr.set_coarse_x(coarse_x + 1, Time::Immediate);
            }
        }
    }

    pub fn increment_scrolly(&self) {
        if self.enabled() {
            let mut addr = self.addr.get();
            match addr.get_fine_y().checked_add(1) {
                Some(y) => addr.set_fine_y(y, Time::Immediate),
                None => {
                    addr.set_fine_y(PixelCoord::Z, Time::Immediate);
                    match addr.get_coarse_y().get() {
                        29 => {
                            addr.set_coarse_y(TileCoord::Z, Time::Immediate);
                            let nt = addr.get_nametable();
                            addr.set_nametable(nt.flip_y(), Time::Immediate);
                        }
                        31 => addr.set_coarse_y(TileCoord::Z, Time::Immediate),
                        _ => (),
                    }
                }
            }
        }
    }

    pub fn transfer_x(&self) {
        if self.enabled() {
            let mut addr = self.addr.get();
            addr.transfer_x();
            self.addr.set(addr);
        }
    }

    pub fn transfer_y(&self) {
        if self.enabled() {
            let mut addr = self.addr.get();
            addr.transfer_y();
            self.addr.set(addr);
        }
    }
}

impl Default for Color {
    fn default() -> Self { Color::Normal }
}
impl Default for Show {
    fn default() -> Self { Show::Show }
}
impl Default for Emphasis {
    fn default() -> Self { Emphasis::Off }
}

impl From<Status> for u8 {
    #[inline]
    fn from(s: Status) -> u8 {
        ((s.overflow as u8) << 5) | ((s.zero_hit as u8) << 6) | ((s.vblank as u8) << 7)
    }
}

#[inline]
pub fn test_bit<T>(val: u8, bit: u8, on: T, off: T) -> T {
    if val & (1 << bit) != 0 {
        on
    } else {
        off
    }
}

impl From<u8> for Mask {
    #[inline]
    fn from(bits: u8) -> Mask {
        Mask {
            color: test_bit(bits, 0, Color::Greyscale, Color::Normal),
            background_left: test_bit(bits, 1, Show::Show, Show::Hide),
            sprites_left: test_bit(bits, 2, Show::Show, Show::Hide),
            background: test_bit(bits, 3, Show::Show, Show::Hide),
            sprites: test_bit(bits, 4, Show::Show, Show::Hide),
            red: test_bit(bits, 5, Emphasis::On, Emphasis::Off),
            green: test_bit(bits, 6, Emphasis::On, Emphasis::Off),
            blue: test_bit(bits, 7, Emphasis::On, Emphasis::Off),
        }
    }
}

impl From<u8> for Control {
    #[inline]
    fn from(bits: u8) -> Control {
        Control {
            base_nt: new_wrapping!(NTAddr, bits),
            vram_inc: test_bit(bits, 2, 32, 1),
            sprite_table: test_bit(bits, 3, PTIdx::Right, PTIdx::Left),
            bg_table: test_bit(bits, 4, PTIdx::Right, PTIdx::Left),
            sprite_height: test_bit(bits, 5, 16, 8),
            interrupt: test_bit(bits, 7, true, false),
        }
    }
}
