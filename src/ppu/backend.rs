use minifb::{Window, WindowOptions, Result, Scale, ScaleMode};
use image::{ImageBuffer, Luma, GenericImage, Bgra, DynamicImage};

use crate::memory::Cartridge;

use super::{Vram, Point, PixelCoord};
use super::pattern::{PatternTable, PTIdx, PatternTile};

pub struct Ppu {
    pub win: Window,
    fb: ImageBuffer<Luma<u8>, Vec<u8>>,
}

impl Ppu {
    pub fn open() -> Result<Self> {
    	let win = Window::new("Patterns", 256, 128, WindowOptions{ resize: true, scale: Scale::FitScreen, scale_mode: ScaleMode::AspectRatioStretch, ..Default::default()})?;
    	Ok(Self {
    		win,
    		fb: ImageBuffer::new(256, 128),
    	})
    }

    pub fn show_patterns(&mut self, cart: &Cartridge<'_>) -> Result<()> {
    	let left = cart.get_pattern_table(PTIdx::Left);
    	for x in PatternTile::Z.. {
    		for y in PatternTile::Z.. {
    			let mut dst = self.fb.sub_image(u32::from(x.get()) * 8, u32::from(y.get()) * 8, 8, 8);
    			let tile = left.get_tile(Point{x, y});

    			for x in PixelCoord::Z.. {
    				for y in PixelCoord::Z.. {
    					let color = tile.get_pixel(Point{x, y});
    					dst.put_pixel(x.get().into(), y.get().into(), Luma([color.get()]))
    				}
    			}
    		}
    	}

    	let right = cart.get_pattern_table(PTIdx::Right);
    	for x in PatternTile::Z.. {
    		for y in PatternTile::Z.. {
    			let mut dst = self.fb.sub_image(u32::from(x.get()) * 8 + 128, u32::from(y.get()) * 8, 8, 8);
    			let tile = right.get_tile(Point{x, y});

    			for x in PixelCoord::Z.. {
    				for y in PixelCoord::Z.. {
    					let color = tile.get_pixel(Point{x, y});
    					dst.put_pixel(x.get().into(), y.get().into(), Luma([color.get()]))
    				}
    			}
    		}
    	}

    	let expanded = self.fb.clone().expand_palette(&[(0x00,0x00,0x00), (0x80,0x80,0x80), (0xC0,0xC0,0xC0), (0xF0,0xF0,0xF0)], None);
    	let converted = DynamicImage::ImageRgba8(expanded).to_bgra8();
    	eprintln!("Updating window");
    	self.win.update_with_buffer(unsafe { &*(converted.as_chunks().0 as *const[[u8; 4]] as *const[u32]) }, 256, 128)
    }
}

