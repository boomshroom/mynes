use std::env;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use memmap::Mmap;
use mynes::ppu::pattern::PTIdx;
use mynes::{Nes, Rom};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os().skip(1).next();
    let path: &Path = path
        .as_ref()
        .map(|p| p.as_ref())
        .unwrap_or("./tests/roms/instr_test-v5/all_instrs.nes".as_ref());
    let rom = unsafe { Mmap::map(&File::open(path)?)? };
    let rom = Rom::parse(&rom[..]).unwrap();

    // println!("{:#?}", rom.header);
    let mut nes = Nes::new(&rom);
    //nes.set_pc(0xC000);
    nes.run().unwrap();

    //let mut ppu = Ppu::open()?;
    //ppu.show_background(&nes.bus.ppu.vram[0], nes.bus.cartridge.get_pattern_table(PTIdx::Left))?;

    //while ppu.win.is_open() {
    //    ppu.update()?;
    //}

    Ok(())
}
