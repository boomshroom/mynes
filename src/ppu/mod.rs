use bounded_integer::bounded_integer;

const NAMETABLE_LEN : usize = 0x400;
pub type Nametable = [Cell<u8>; NAMETABLE_LEN];

bounded_integer! {
	pub struct VAddr { 0..0x4000 }
	enum NTAddr { 0..4 }
	enum VReg { 0..8 }
}

pub struct Vram<'a> {
	oam: OAM,
	palette: [u8; 64],
	nametables: [&'a Nametable; 4],
	registers: Registers,
}

#[derive(Default,Debug,Clone)]
pub struct Registers {
	control: u8,
	mask: u8,
	status: Status,
	oam_addr: u8,
	oam_data: u8,
	scroll: (u8, u8),
	scroll_latch: ScrollLatch,
	addr: AddrReg,
}

pub struct Oam ([u8; 256]);

pub struct Sprite {
	y: u8,
	tile: u8,
	attr: u8,
	x: u8,
}

impl Oam {
	fn write_byte(&mut self, val: u8, idx: u8) {
		self.0[usize::from(idx)] = val;
	}

	fn get_sprite(&self, idx: u8) -> Sprite {
		let (chunks, extra) = self.0.as_chunks::<4>();
		debug_assert_eq!(extra, &[]);
		let block = chunks[usize::from(idx)];
		Sprite {
			y: block[0],
			tile: block[1],
			attr: block[2],
			x: block[3],
		}
	}
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
enum ScrollLatch {
	X,
	Y,
}

struct AddrReg {
	address: VAddr,
	latch: AddrLatch,
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
enum AddrLatch {
	High,
	Low,
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

#[derive(Debug,Copy,Clone)]
struct Status {
	overflow: bool,
	zero_hit: bool,
	vblank: bool,
}

#[derive(Debug,Copy,Clone)]
struct Mask {
	color: Color,
	background_left: Show,
	sprites_left: Show,
	background: Show,
	sprites: Show,
	red: Emphasis,
	green: Emphasis,
	blue: Emphasis,
}

impl Registers {
	fn write_scroll(&mut self, val: u8) {
		match self.scroll_latch {
			ScrollLatch::X => {
				self.scroll.0 = val;
				self.scroll_latch = ScrollLatch::Y,
			},
			ScrollLatch::Y => {
				self.scroll.1 = val;
				self.scroll_latch = ScrollLatch::X,
			}
		}
	}
}

impl AddrReg {
	fn write_addr(&mut self, val: u8) {
		match self.latch {
			AddrLatch::High => {
				self.address = self.address & 0xFF | (u16::from(val) & 0x3F) << 8;
				self.latch = AddrLatch::Low,
			},
			AddrLatch::Low => {
				self.address = self.address & 0x3F00 | u16::from(val);
				self.latch = AddrLatch::High,
			}
		}
	}
}

impl From<Status> for u8 {
	#[inline]
	fn from(s: Status) -> u8 {
		s.overflow as u8 << 5 | s.zero_hit as u8 << 6 | s.vblank as u8 << 7
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
			color: test_bit(bits, 0, Color::Greyscale, Color::Color),
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