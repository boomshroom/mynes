use std::io;

use mynes::Nes;
use mynes::Rom;

fn test_rom(rom: &[u8]) -> io::Result<()> {
    let rom = Rom::parse(&rom[..]).unwrap();
    let mut nes = Nes::new(&rom);
    let e = nes.run();

    //eprintln!("{:#?}", nes.cpu);
    let status = nes.get_mem(0x6000);
    if status == 0x81 {
        todo!("Reset");
    }
    eprintln!("({:02x})", status);

    let mut msg = String::new();
    for addr in 0x6004.. {
        match nes.get_mem(addr) {
            0 | 0xff => break,
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

macro_rules! test_file {
    ($name:ident($path:expr)) => {
        #[test]
        fn $name() -> io::Result<()> {
            test_rom(include_bytes!(concat!("roms/", $path, ".nes")))
        }
    };

    ([i]$name:ident($path:expr)) => {
        #[test]
        #[ignore]
        fn $name() -> io::Result<()> {
            test_rom(include_bytes!(concat!("roms/", $path, ".nes")))
        }
    };
}

macro_rules! instr_test {
    ($name:ident($id:literal)) => {
        test_file!($name(concat!(
            "instr_test-v5/rom_singles/",
            $id,
            "-",
            stringify!($name),
        )));
    };
}

instr_test!(basics("01"));
instr_test!(implied("02"));
instr_test!(immediate("03"));
instr_test!(zero_page("04"));
instr_test!(zp_xy("05"));
instr_test!(absolute("06"));
instr_test!(abs_xy("07"));
instr_test!(ind_x("08"));
instr_test!(ind_y("09"));
instr_test!(branches("10"));
instr_test!(stack("11"));
instr_test!(jmp_jsr("12"));
instr_test!(rts("13"));
instr_test!(rti("14"));
instr_test!(brk("15"));
instr_test!(special("16"));

test_file!(official("instr_test-v5/official_only"));
test_file!(all("instr_test-v5/all_instrs"));

test_file!(timing("nes-test-roms/instr_timing/instr_timing"));
test_file!(instr_timing(
    "nes-test-roms/instr_timing/rom_singles/1-instr_timing"
));
test_file!(branch_timing(
    "nes-test-roms/instr_timing/rom_singles/2-branch_timing"
));

test_file!(apu_test("nes-test-roms/apu_test/apu_test"));
test_file!([i]len_ctr("nes-test-roms/apu_test/rom_singles/1-len_ctr"));
test_file!([i]len_table("nes-test-roms/apu_test/rom_singles/2-len_table"));
test_file!([i]irq_flag("nes-test-roms/apu_test/rom_singles/3-irq_flag"));
test_file!([i]jitter("nes-test-roms/apu_test/rom_singles/4-jitter"));
test_file!([i]len_timing("nes-test-roms/apu_test/rom_singles/5-len_timing"));
test_file!([i]irq_flag_timing("nes-test-roms/apu_test/rom_singles/6-irq_flag_timing"));
test_file!([i]dmc_basics("nes-test-roms/apu_test/rom_singles/7-dmc_basics"));
test_file!([i]dmc_rates("nes-test-roms/apu_test/rom_singles/8-dmc_rates"));

test_file!([i]apu_4015_cleared("nes-test-roms/apu_reset/4015_cleared"));
test_file!([i]apu_4017_timing("nes-test-roms/apu_reset/4017_timing"));
test_file!([i]apu_4017_written("nes-test-roms/apu_reset/4017_written"));
test_file!([i]apu_irq_flag_cleared("nes-test-roms/apu_reset/irq_flag_cleared"));
test_file!([i]apu_len_ctrs_enabled("nes-test-roms/apu_reset/len_ctrs_enabled"));
test_file!([i]apu_works_immediately("nes-test-roms/apu_reset/works_immediately"));
