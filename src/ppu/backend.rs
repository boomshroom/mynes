use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use image::{Bgra, DynamicImage, GenericImage, ImageBuffer, Luma};
use minifb::{Result, Scale, ScaleMode, Window, WindowOptions};

use super::pattern::{PTIdx, PatternTableRef, PatternTile};
use super::{Nametable, PixelCoord, Point, Vram};
use crate::memory::Cartridge;

pub struct Ppu {
    pub win: Window,
    fb: Arc<Mutex<ImageBuffer<Bgra<u8>, Vec<u8>>>>,
}

impl Ppu {
    pub fn open(fb: Arc<Mutex<ImageBuffer<Bgra<u8>, Vec<u8>>>>) -> Arc<AtomicBool> {
        let running = Arc::new(AtomicBool::new(true));
        let res = running.clone();

        thread::spawn(move || {
            let mut win = Window::new("Background", 256, 240, WindowOptions {
                resize: true,
                scale: Scale::FitScreen,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..Default::default()
            }).unwrap();

            while win.is_open() {
                thread::sleep(Duration::from_secs(1) / 60);
                let fb = fb.lock().unwrap();
                win.update_with_buffer(
                    unsafe { &*(fb.as_chunks().0 as *const [[u8; 4]] as *const [u32]) },
                    256,
                    240,
                ).unwrap()
            }
            running.store(false, Ordering::Relaxed)
        });
        res
    }
}
