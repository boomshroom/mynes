use bounded_integer::bounded_integer;

pub struct Oam([Sprite; 64]);

bounded_integer!(pub struct OamIdx { 0..64 });

#[derive(Debug, Copy, Clone, Default)]
pub struct Sprite {
    y: u8,
    tile: u8,
    attr: u8,
    x: u8,
}

impl Oam {
    pub fn new() -> Self { Oam([Sprite::default(); 64]) }

    pub fn write_byte(&mut self, val: u8, idx: u8) {
        let sprite = &mut self.0[usize::from(idx / 4)];
        match idx % 4 {
            0 => sprite.y = val,
            1 => sprite.tile = val,
            2 => sprite.attr = val,
            3 => sprite.x = val,
            _ => unreachable!(),
        }
    }

    pub fn get_sprite(&self, idx: OamIdx) -> Sprite { self.0[usize::from(idx.get())] }
}
