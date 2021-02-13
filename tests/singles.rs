use memmap::Mmap;
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

use mynes::Nes;
use mynes::Rom;

fn test_rom(name: &str) -> io::Result<()> {
    let rom = unsafe { Mmap::map(&File::open(name)?)? };
    let rom = Rom::parse(&rom[..]).unwrap();
    let mut nes = Nes::new(&rom);
    let e = nes.run();

    //eprintln!("{:#?}", nes.cpu);
    eprintln!("({:02x})", nes.get_mem(0x6000));

    let mut msg = String::new();
    for addr in 0x6004.. {
        match nes.get_mem(addr) {
            0 | 0xff | b'0' => break,
            ch => msg.push(ch as char),
        }
    }

    eprintln!("{}", msg);

    if let Err(e) = e {
        panic!("{}", e);
    }

    assert_eq!(nes.get_mem(0x6000), 0);

    Ok(())
}

macro_rules! single {
    ($name:ident($id:literal)) => {
        #[test]
        fn $name() -> io::Result<()> {
            test_rom(concat!(
                "instr_test-v5/rom_singles/",
                $id,
                "-",
                stringify!($name),
                ".nes"
            ))
        }
    };
}

single!(basics("01"));
single!(implied("02"));
single!(immediate("03"));
single!(zero_page("04"));
single!(zp_xy("05"));
single!(absolute("06"));
single!(abs_xy("07"));
single!(ind_x("08"));
single!(ind_y("09"));
single!(branches("10"));
single!(stack("11"));
single!(jmp_jsr("12"));
single!(rts("13"));
single!(rti("14"));
single!(brk("15"));
single!(special("16"));

#[test]
fn official() -> io::Result<()> {
    test_rom("instr_test-v5/official_only.nes")
}

#[test]
fn all() -> io::Result<()> {
    test_rom("instr_test-v5/all_instrs.nes")
}

#[test]
fn all_timing() -> io::Result<()> {
    test_rom("instr_timing/instr_timing.nes")
}

#[test]
fn instr_timing() -> io::Result<()> {
    test_rom("instr_timing/rom_singles/1-instr_timing.nes")
}

#[test]
fn branch_timing() -> io::Result<()> {
    test_rom("instr_timing/rom_singles/2-branch_timing.nes")
}

#[test]
fn nestest() -> io::Result<()> {
    let rom = unsafe { Mmap::map(&File::open("nestest.nes")?)? };
    let rom = Rom::parse(&rom[..]).unwrap();
    let mut nes = Nes::new(&rom);
    nes.set_pc(0xc000);
    let e = nes.run();

    //eprintln!("{:#?}", nes.cpu);
    let result = [nes.get_mem(2), nes.get_mem(3)];
    assert_eq!(result, [0, 0], "[{:02x} {:02x}]", result[0], result[1]);

    if let Err(e) = e {
        panic!("{}", e);
    }

    Ok(())
}
