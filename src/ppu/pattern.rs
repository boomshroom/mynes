use bounded_integer::bounded_integer;

use super::{PixelCoord, Point};

pub struct PatternTable ([u8; 0x1000]);
pub struct PatternTableRef<'a> (&'a [u8; 0x1000]);

pub struct TileData {
	front: u64,
	back: u64,
}

bounded_integer!(enum PatternTile { 0..16 });
bounded_integer!(enum TileColor { 0..4 });

enum Bit {
	Low,
	High,
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum PTIdx {
	Left,
	Right,
}

impl Default for PTIdx {
	fn default() -> Self {
		PTIdx::Left
	}
}

impl PatternTable {
	fn get_row(&self, tile: Point<PatternTile>, row: PixelCoord, bit: Bit) -> u8 {
		let (rows, rest) = self.0.as_chunks::<256>();
		debug_assert_eq!(rest, &[] as &[u8]);
		let tile_row = &rows[usize::from(tile.y.get())];

		let (cells, rest) = tile_row.as_chunks::<16>();
		debug_assert_eq!(rest, &[] as &[u8]);

		let cell = &cells[usize::from(tile.x.get())];
		match bit {
			Bit::Low => cell[usize::from(row.get())],
			Bit::High => cell[usize::from(row.get() | 0x8)],
		}		
	}

	fn get_tile(&self, tile: Point<PatternTile>) -> TileData {
		let (rows, rest) = self.0.as_chunks::<256>();
		debug_assert_eq!(rest, &[] as &[u8]);
		let row = &rows[usize::from(tile.y.get())];

		let (cells, rest) = row.as_chunks::<16>();
		debug_assert_eq!(rest, &[] as &[u8]);

		TileData::from(u128::from_le_bytes(cells[usize::from(tile.x.get())]))
	}
}

impl TileData {
	fn get_pixel(&self, x: PixelCoord, y: PixelCoord) -> TileColor {
		let bit = x.get() + y.get() * 8;
		let lsb = (self.front & 1<<bit != 0) as u8;
		let msb = (self.back & 1<<bit != 0) as u8;
		TileColor::new(lsb | (msb << 1)).unwrap()
	}
}

impl From<u128> for TileData {
	fn from(val: u128) -> TileData {
		let front = (val & ((1 << 64) - 1)) as u64;
		let back = (val >> 64) as u64;
		TileData { front, back }
	}
}

impl From<TileData> for u128 {
	fn from(TileData{ front, back }: TileData) -> u128 {
		front as u128 | ((back as u128) << 64)
	}
}

