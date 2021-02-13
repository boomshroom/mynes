use std::env;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;

use memmap::Mmap;
use mynes::{Nes, Rom};

fn main() -> Result<()> {
    let path = env::args_os().skip(1).next();
    let path: &Path = path
        .as_ref()
        .map(|p| p.as_ref())
        .unwrap_or("./nestest.nes".as_ref());
    let rom = unsafe { Mmap::map(&File::open(path)?)? };
    let rom = Rom::parse(&rom[..]).unwrap();

    File::create("dump.prg")?.write_all(rom.prg)?;

    println!("{:#?}", rom.header);
    println!("{:?}", &rom.prg[..16]);
    let mut nes = Nes::new(&rom);
    //nes.set_pc(0xC000);
    nes.run().unwrap();
    Ok(())
}
