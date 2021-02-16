use super::VAddr;
use super::NTAddr;
use super::loopy::AddrReg;
use super::pattern::PTIdx;

#[derive(Default,Debug,Clone)]
pub struct Registers {
	pub control: Control,
	pub mask: Mask,
	pub status: Status,
	oam_addr: u8,
	oam_data: u8,
	pub addr: AddrReg,
}

#[derive(Debug,Copy,Clone,Default)]
pub struct Control {
	base_nt: NTAddr,
	vram_inc: u16,
	sprite_table: PTIdx,
	bg_table: PTIdx,
	sprite_height: u8,
	interrupt: bool,
}

#[derive(Debug,Clone,Copy,Default)]
pub struct Status {
	overflow: bool,
	zero_hit: bool,
	vblank: bool,
}

#[derive(Debug,Copy,Clone,Default)]
pub struct Mask {
	color: Color,
	background_left: Show,
	sprites_left: Show,
	background: Show,
	sprites: Show,
	red: Emphasis,
	green: Emphasis,
	blue: Emphasis,
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
enum Color {
	Normal,
	Greyscale,
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
enum Show {
	Show,
	Hide,
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
enum Emphasis {
	On,
	Off,
}

impl Registers {
	pub fn clear_vblank(&mut self) {
		self.status.vblank = false;
	}

	pub fn advance_vaddr(&mut self) -> VAddr {
		self.addr.advance(self.control.vram_inc)
	}

	pub fn set_control(&mut self, val: u8) {
		let reg = val.into();
		self.control = reg;
		self.addr.set_nametable(reg.base_nt);
	}
}

impl Default for Color { fn default() -> Self { Color::Normal } }
impl Default for Show { fn default() -> Self { Show::Show } }
impl Default for Emphasis { fn default() -> Self { Emphasis::Off } }

impl From<Status> for u8 {
	#[inline]
	fn from(s: Status) -> u8 {
		((s.overflow as u8) << 5) | ((s.zero_hit as u8) << 6) | ((s.vblank as u8) << 7)
	}
}

#[inline]
fn test_bit<T>(val: u8, bit: u8, on: T, off: T) -> T {
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
			base_nt: NTAddr::new_wrapping(bits),
			vram_inc: test_bit(bits, 2, 32, 1),
			sprite_table: test_bit(bits, 3, PTIdx::Right, PTIdx::Left),
			bg_table: test_bit(bits, 4, PTIdx::Right, PTIdx::Left),
			sprite_height: test_bit(bits, 5, 16, 8),
			interrupt: test_bit(bits, 7, true, false),
		}
	}
}