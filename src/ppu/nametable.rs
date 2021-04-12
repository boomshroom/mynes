use std::cell::Cell;

pub const NAMETABLE_LEN: usize = 0x400;
#[derive(Debug, Clone)]
pub struct Nametable([Cell<u8>; NAMETABLE_LEN]);

const EMPTY_CELL : Cell<u8> = Cell::new(0);

impl Nametable {
    pub const fn new() -> Self { Nametable([EMPTY_CELL; NAMETABLE_LEN]) }

    pub fn read(&self, idx: u16) -> u8 { self.0[usize::from(idx)].get() }

    pub fn write(&self, idx: u16, val: u8) { self.0[usize::from(idx)].set(val); }

    pub fn attr_table(&self) -> &[Cell<u8>; 64] { todo!() }
}
