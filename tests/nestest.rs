use std::io;

use mynes::Nes;
use mynes::Rom;


#[test]
fn nestest() -> io::Result<()> {
    let rom = Rom::parse(include_bytes!("roms/nestest.nes")).unwrap();
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
