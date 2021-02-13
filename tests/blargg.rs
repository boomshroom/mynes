use std::io;

use mynes::Nes;
use mynes::Rom;

fn test_rom(rom: &[u8]) -> io::Result<()> {
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
            test_rom(include_bytes!(concat!(
                "roms/instr_test-v5/rom_singles/",
                $id,
                "-",
                stringify!($name),
                ".nes"
            )))
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
    test_rom(include_bytes!("roms/instr_test-v5/official_only.nes"))
}

#[test]
fn all() -> io::Result<()> {
    test_rom(include_bytes!("roms/instr_test-v5/all_instrs.nes"))
}

#[test]
fn all_timing() -> io::Result<()> {
    test_rom(include_bytes!("roms/nes-test-roms/instr_timing/instr_timing.nes"))
}

#[test]
fn instr_timing() -> io::Result<()> {
    test_rom(include_bytes!("roms/nes-test-roms/instr_timing/rom_singles/1-instr_timing.nes"))
}

#[test]
fn branch_timing() -> io::Result<()> {
    test_rom(include_bytes!("roms/nes-test-roms/instr_timing/rom_singles/2-branch_timing.nes"))
}
