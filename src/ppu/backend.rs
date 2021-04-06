use std::time::Duration;

use minifb::{Window, WindowOptions, Result, Scale, ScaleMode};
use image::{ImageBuffer, Luma, GenericImage, Bgra, DynamicImage};

use crate::memory::Cartridge;

use super::{Vram, Point, PixelCoord, Nametable};
use super::pattern::{PatternTableRef, PTIdx, PatternTile};

pub struct Ppu {
    pub win: Window,
    fb: ImageBuffer<Luma<u8>, Vec<u8>>,
}

impl Ppu {
    pub fn open() -> Result<Self> {
    	let mut win = Window::new("Background", 256, 240, WindowOptions{ resize: true, scale: Scale::FitScreen, scale_mode: ScaleMode::AspectRatioStretch, ..Default::default()})?;
    	win.limit_update_rate(Some(Duration::from_secs(1) / 60));
        Ok(Self {
    		win,
    		fb: ImageBuffer::new(256, 240),
    	})
    }

    pub fn update(&mut self) -> Result<()> {
        let expanded = self.fb.clone().expand_palette(&[(0x00,0x00,0x00), (0x80,0x80,0x80), (0xC0,0xC0,0xC0), (0xF0,0xF0,0xF0)], None);
        let converted = DynamicImage::ImageRgba8(expanded).to_bgra8();
        eprintln!("Updating window");
        self.win.update_with_buffer(unsafe { &*(converted.as_chunks().0 as *const[[u8; 4]] as *const[u32]) }, 256, 240)
    }

    pub fn show_background(&mut self, table: &Nametable, patterns: PatternTableRef<'_>) -> Result<()> {
        for y in 0..30 {
            for x in 0..32 {
                let mut dst = self.fb.sub_image(u32::from(x) * 8, u32::from(y) * 8, 8, 8);
                let idx = table.read(y * 32 + x);
                let tile = patterns.get_tile(idx.into());
                for x in PixelCoord::MIN..=PixelCoord::MAX {
                    for y in PixelCoord::MIN..=PixelCoord::MAX {
                        let color = tile.get_pixel(Point{x, y});
                        dst.put_pixel((PixelCoord::MAX_VALUE - x.get()).into(), y.get().into(), Luma([color.get()]))
                    }
                }
            }
        }
        self.update()
    }

    pub fn show_patterns(&mut self, cart: &Cartridge<'_>) -> Result<()> {
    	let left = cart.get_pattern_table(PTIdx::Left);
    	for x in PatternTile::MIN..=PatternTile::MAX {
    		for y in PatternTile::MIN..=PatternTile::MAX {
    			let mut dst = self.fb.sub_image(u32::from(x.get()) * 8, u32::from(y.get()) * 8, 8, 8);
    			let tile = left.get_tile(Point{x, y});

    			for x in PixelCoord::MIN..=PixelCoord::MAX {
    				for y in PixelCoord::MIN..=PixelCoord::MAX {
    					let color = tile.get_pixel(Point{x, y});
    					dst.put_pixel((PixelCoord::MAX_VALUE - x.get()).into(), y.get().into(), Luma([color.get()]))
    				}
    			}
    		}
    	}

    	let right = cart.get_pattern_table(PTIdx::Right);
    	for x in PatternTile::MIN..=PatternTile::MAX {
    		for y in PatternTile::MIN..=PatternTile::MAX {
    			let mut dst = self.fb.sub_image(u32::from(x.get()) * 8 + 128, u32::from(y.get()) * 8, 8, 8);
    			let tile = right.get_tile(Point{x, y});

    			for x in PixelCoord::MIN..=PixelCoord::MAX {
    				for y in PixelCoord::MIN..=PixelCoord::MAX {
    					let color = tile.get_pixel(Point{x, y});
    					dst.put_pixel((PixelCoord::MAX_VALUE - x.get()).into(), y.get().into(), Luma([color.get()]))
    				}
    			}
    		}
    	}

        self.update()
    }
}

