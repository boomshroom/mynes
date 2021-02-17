use std::env;
use std::fs::File;
use std::error::Error;
use std::path::Path;

use memmap::Mmap;
use mynes::{Nes, Rom};
use mynes::ppu::backend::Ppu;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os().skip(1).next();
    let path: &Path = path
        .as_ref()
        .map(|p| p.as_ref())
        .unwrap_or("./tests/roms/nestest.nes".as_ref());
    let rom = unsafe { Mmap::map(&File::open(path)?)? };
    let rom = Rom::parse(&rom[..]).unwrap();

    println!("{:#?}", rom.header);
    let mut nes = Nes::new(&rom);
    nes.set_pc(0xC000);
    nes.run().unwrap();

    let mut ppu = Ppu::open()?;
    ppu.show_patterns(&nes.bus.cartridge)?;

    while ppu.win.is_open() {
        ppu.win.update()
    }

    Ok(())
}
