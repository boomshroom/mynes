use std::rc::Rc;
use std::cell::Cell;
use std::sync::{Mutex, Arc};
use genawaiter::stack::Co;
use genawaiter::stack::let_gen_using;
use genawaiter::GeneratorState;

use image::{ImageBuffer, Bgra};
use super::{Registers, VAddr, PaletteIdx, regs::Show, palette::TileColor};

pub struct FrameBuffer {
    buffer: Arc<Mutex<ImageBuffer<Bgra<u8>, Vec<u8>>>>,
} 

macro_rules! yield_ {
    ($op:expr, $co:expr) => {
        $co.yield_($op).await
    };
}

pub enum VOp {
    Nop,
    Fetch(VAddr),
    Nmi,
}

pub struct DrawCommand {
    pub point: (u32, u32),
    pub tile: TileColor,
    pub palette: PaletteIdx,
}

#[derive(Debug, Clone, Default)]
pub struct LiveRender {
    attrib: Cell<PaletteIdx>,
    tile_low: Cell<u8>,
    tile_high: Cell<u8>,
    attrib_shift: Cell<Shift>,
    pattern_shift: Cell<Shift>,
}

impl LiveRender {
    fn load_shifters(&self) {
        use super::regs::test_bit;

        let attrib = self.attrib.get().get();
        self.pattern_shift.update(|sh| sh.load(self.tile_low.get(), self.tile_high.get()));
        self.attrib_shift.update(|sh| sh.load(test_bit(attrib, 0, 0xFF, 0), test_bit(attrib, 1, 0xFF, 0)));
    }

    fn update(&self, mask: bool) {
        self.pattern_shift.update(|sh| sh.update(mask));
        self.attrib_shift.update(|sh| sh.update(mask));
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Shift {
    low: u16,
    high: u16,
}

impl Shift {
    fn load(mut self, low: u8, high: u8) -> Self {
        self.low = (self.low & 0xFF00) | u16::from(low);
        self.high = (self.high & 0xFF00) | u16::from(high);
        self
    }

    fn update(mut self, mask: bool) -> Self {
        if mask {
            self.low <<= 1;
            self.high <<= 1;
        }
        self
    }
}

impl FrameBuffer {
    pub fn new() -> Self {
        FrameBuffer{
            buffer: Arc::new(Mutex::new(ImageBuffer::new(256, 230)))
        }
    }

    pub async fn clock(regs: Rc<Registers>, co: Co<'_, (VOp, Option<DrawCommand>), u8>) -> ! {
        let shared = LiveRender::default();

        let mut byte = 0_u8;

        for y in (-1_i32..261).cycle().skip(1) {
            let_gen_using!(scanline, |co| Self::scanline(regs.clone(), &shared, co));

            for x in (if y == 0 { 1_u32 } else { 0 })..341 {
                let cmd = match y {
                    -1 ..= 239 => {
                        let mut cmd = VOp::Nop;
                        match x {
                            1 if y == -1 => regs.set_vblank(false),
                            2 ..= 257 | 321 ..= 340 => {
                                if x < 338 {
                                    shared.update(regs.enabled());
                                }

                                cmd = match scanline.resume_with(byte) {
                                    GeneratorState::Yielded(cmd) => cmd,
                                    GeneratorState::Complete(never) => never,
                                };
                                if x == 256 {
                                    regs.increment_scrolly();
                                } else if x == 257 {
                                    shared.load_shifters();
                                    regs.transfer_x();
                                }
                            },
                            280 ..= 304 if y == -1 => regs.transfer_y(),
                            _ => (),
                        }
                        if (x, y) == (1, -1) { regs.set_vblank(false); }
                        cmd
                    },
                    241 if x == 1 => {
                        regs.set_vblank(true);
                        match regs.interrupt_enabled() {
                            true => VOp::Nmi,
                            false => VOp::Nop,
                        }
                    },
                    _ => VOp::Nop,
                };

                let draw = if (1..=256).contains(&x) && (0..240).contains(&y) && regs.mask.get().background == Show::Show {
                    let addr = regs.addr.get();

                    let bit_mux = 0x8000 >> addr.get_fine_x().get();

                    let pattern = shared.pattern_shift.get();
                    let p0_pixel = (pattern.low & bit_mux) > 0;
                    let p1_pixel = (pattern.high & bit_mux) > 0;

                    let tile = TileColor::new(p0_pixel as u8 | (p1_pixel as u8) << 1).unwrap();

                    let attrib = shared.attrib_shift.get();
                    let pal_0 = (attrib.low & bit_mux) > 0;
                    let pal_1 = (attrib.high & bit_mux) > 0;

                    let palette = PaletteIdx::new(pal_0 as u8 | (pal_1 as u8) << 1).unwrap();

                    Some(DrawCommand{ point: (x - 1, y as u32), tile, palette })
                } else {
                    None
                };

                byte = yield_!((cmd, draw), co);
            }
        }
        unreachable!()
    }

    pub async fn scanline(regs: Rc<Registers>, shared: &LiveRender, co: Co<'_, VOp, u8>) -> ! {
        let vreg = &regs.addr;
        loop {
            shared.load_shifters();

            let tile_id = yield_!(
                VOp::Fetch(VAddr::new(0x2000 | (vreg.get().get_addr().get() & 0xFFF)).unwrap()),
                co
            );
            yield_!(VOp::Nop, co);

            let mut attrib : u8 = yield_!(
                VOp::Fetch(
                    VAddr::new(
                        0x23C0
                            | (u16::from(vreg.get().get_nametable().get()) << 10_u8)
                            | (u16::from((vreg.get().get_coarse_x().get()) >> 2_u8) << 3_u8)
                            | (u16::from(vreg.get().get_coarse_y().get()) >> 2_u8)
                    )
                    .unwrap()
                ),
                co
            );
            if vreg.get().get_coarse_y().get() & 0x02 != 0 {
                attrib >>= 4;
            }
            if vreg.get().get_coarse_x().get() & 0x02 != 0 {
                attrib >>= 4;
            }
            shared.attrib.set(new_wrapping!(PaletteIdx, attrib));
            yield_!(VOp::Nop, co);

            shared.tile_low.set(yield_!(
                VOp::Fetch(
                    VAddr::new(
                        ((regs.control.get().bg_table as u16) << 12_u8)
                            | (u16::from(tile_id) << 4_u8)
                            | u16::from(vreg.get().get_fine_y().get())
                    )
                    .unwrap()
                ),
                co
            ));
            yield_!(VOp::Nop, co);

            shared.tile_high.set(yield_!(
                VOp::Fetch(
                    VAddr::new(
                        ((regs.control.get().bg_table as u16) << 12_u8)
                            | (u16::from(tile_id) << 4_u8)
                            | u16::from(vreg.get().get_fine_y().get())
                    )
                    .unwrap() + 8
                ),
                co
            ));
            regs.increment_scrollx();
            yield_!(VOp::Nop, co);
        }
    }
}
