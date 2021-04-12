use super::{NTAddr, PixelCoord, Point, TileCoord, VAddr};

#[derive(Debug, Copy, Clone)]
pub struct AddrReg {
    address: u16,
    temp: u16,
    fine_x: PixelCoord,
    latch: AddrLatch,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum AddrLatch {
    High,
    Low,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Time {
    Delayed,
    Immediate,
}

impl Default for AddrReg {
    fn default() -> Self { Self::new() }
}

impl AddrReg {
    pub fn new() -> Self {
        AddrReg {
            address: 0,
            temp: 0,
            fine_x: PixelCoord::new(0).unwrap(),
            latch: AddrLatch::High,
        }
    }

    pub fn reset(&mut self) { *self = Self::new(); }

    fn reg(&mut self, t: Time) -> &mut u16 {
        match t {
            Time::Delayed => &mut self.temp,
            Time::Immediate => &mut self.address,
        }
    }

    pub fn write_addr(mut self, val: u8) -> Self{
        match self.latch {
            AddrLatch::High => {
                self.temp &= 0xFF;
                self.temp |= (u16::from(val) & 0x3F) << 8;
                self.latch = AddrLatch::Low;
            }
            AddrLatch::Low => {
                self.temp &= 0xFF00;
                self.temp |= u16::from(val);
                self.update();
                self.latch = AddrLatch::High;
            }
        };
        self
    }

    pub fn write_scroll(mut self, val: u8, t: Time) -> Self {
        match self.latch {
            AddrLatch::High => {
                self.set_fine_x(new_wrapping!(PixelCoord, val));
                self.set_coarse_x(new_wrapping!(TileCoord, val >> 3), t);
                self.latch = AddrLatch::Low;
            }
            AddrLatch::Low => {
                self.set_fine_y(new_wrapping!(PixelCoord, val), t);
                self.set_coarse_y(new_wrapping!(TileCoord, val >> 3), t);
                self.update();
                self.latch = AddrLatch::High;
            }
        };
        self
    }

    pub fn advance(&mut self, inc: u16) -> VAddr {
        let addr = self.get_addr();
        self.address += inc;
        addr
    }

    pub fn update(&mut self) { self.address = self.temp; }

    pub fn get_addr(&self) -> VAddr { new_wrapping!(VAddr, self.address) }

    pub fn set_coarse_x(&mut self, val: TileCoord, t: Time) {
        let reg = self.reg(t);
        *reg &= 0b1_111_11_11111_00000;
        *reg |= u16::from(val.get());
    }

    pub fn set_coarse_y(&mut self, val: TileCoord, t: Time) {
        let reg = self.reg(t);
        *reg &= 0b1_111_11_00000_11111;
        *reg |= u16::from(val.get()) << 5;
    }

    pub fn set_nametable(&mut self, nt: NTAddr, t: Time) {
        let reg = self.reg(t);
        *reg &= 0b1_111_00_11111_11111;
        *reg |= u16::from(nt.get()) << 10;
    }

    pub fn set_fine_y(&mut self, val: PixelCoord, t: Time) {
        let reg = self.reg(t);
        *reg &= 0b1_000_11_11111_11111;
        *reg |= u16::from(val.get()) << 12;
    }

    pub fn set_fine_x(&mut self, val: PixelCoord) { self.fine_x = val; }

    pub fn get_tile(&self) -> Point<TileCoord> {
        Point {
            x: new_wrapping!(TileCoord, self.address as u8),
            y: new_wrapping!(TileCoord, (self.address >> 5) as u8),
        }
    }

    pub fn transfer_x(&mut self) {
        self.address = (self.address & 0b1_111_10_11111_00000) | (self.temp & 0b0_000_01_00000_11111);
    }

    pub fn transfer_y(&mut self) {
        self.address = (self.address & 0b1_111_01_00000_11111) | (self.temp & 0b0_000_10_11111_00000);
    }

    pub fn get_coarse_x(&self) -> TileCoord { new_wrapping!(TileCoord, self.address as u8) }

    pub fn get_coarse_y(&self) -> TileCoord { new_wrapping!(TileCoord, (self.address >> 5) as u8) }

    pub fn get_nametable(&self) -> NTAddr { new_wrapping!(NTAddr, (self.address >> 10) as u8) }

    pub fn get_pixel(&self) -> Point<PixelCoord> {
        Point {
            x: self.fine_x,
            y: new_wrapping!(PixelCoord, (self.address >> 12) as u8),
        }
    }

    pub fn get_fine_y(&self) -> PixelCoord { new_wrapping!(PixelCoord, (self.address >> 12) as u8) }

    pub fn get_fine_x(&self) -> PixelCoord { self.fine_x }
}
