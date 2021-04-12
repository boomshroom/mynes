use bounded_integer::bounded_integer;
use image::Rgb;

#[derive(Debug, Default)]
pub struct PaletteRam {
    background: u8,
    bg_palettes: [[u8; 3]; 4],
    sprite_palettes: [[u8; 3]; 4],
    unused: [u8; 3],
}

bounded_integer!(pub struct ColorCode { 0..0x40 });
bounded_integer!(pub enum TileColor { 0..4 });
bounded_integer!(pub enum PaletteIdx { 0..4 });

impl Default for PaletteIdx {
    fn default() -> Self { PaletteIdx::Z }
}

impl PaletteRam {
    pub fn write(&mut self, idx: u8, val: u8) {
        let idx = usize::from(idx);
        if idx % 0x10 == 0 {
            self.background = val;
        } else if idx % 4 == 0 {
            self.unused[((idx >> 2) % 4) - 1] = val;
        } else if idx < 0x10 {
            self.bg_palettes[(idx >> 2) % 4][(idx % 4) - 1] = val;
        } else {
            self.sprite_palettes[(idx >> 2) % 4][(idx % 4) - 1] = val;
        }
    }

    pub fn read(&self, idx: u8) -> u8 {
        let idx = usize::from(idx);
        if idx == 0 {
            self.background
        } else if idx % 4 == 0 {
            self.unused[(idx >> 2) % 4]
        } else if idx < 0x10 {
            self.bg_palettes[(idx >> 2) % 4][(idx % 4) - 1]
        } else {
            self.sprite_palettes[(idx >> 2) % 4][(idx % 4) - 1]
        }
    }

    pub fn get_background(&self, tile: TileColor, palette: PaletteIdx) -> ColorCode {
        if tile == TileColor::Z {
            new_wrapping!(ColorCode, self.background)
        } else {
            new_wrapping!(ColorCode, 
                self.bg_palettes[usize::from(palette.get())][usize::from(tile.get() - 1)],
            )
        }
    }

    pub fn get_sprite(&self, tile: TileColor, palette: PaletteIdx) -> ColorCode {
        if tile == TileColor::Z {
            new_wrapping!(ColorCode, self.background)
        } else {
            new_wrapping!(ColorCode, 
                self.sprite_palettes[usize::from(palette.get())][usize::from(tile.get() - 1)],
            )
        }
    }
}

impl ColorCode {
    pub fn as_rgb(self) -> Rgb<u8> { DEFAULT_PALETTE[usize::from(self.get())] }
}

const DEFAULT_PALETTE: [Rgb<u8>; 0x40] = [
    Rgb([84, 84, 84]),
    Rgb([0, 30, 116]),
    Rgb([8, 16, 144]),
    Rgb([48, 0, 136]),
    Rgb([68, 0, 100]),
    Rgb([92, 0, 48]),
    Rgb([84, 4, 0]),
    Rgb([60, 24, 0]),
    Rgb([32, 42, 0]),
    Rgb([8, 58, 0]),
    Rgb([0, 64, 0]),
    Rgb([0, 60, 0]),
    Rgb([0, 50, 60]),
    Rgb([0, 0, 0]),
    Rgb([0, 0, 0]),
    Rgb([0, 0, 0]),
    Rgb([152, 150, 152]),
    Rgb([8, 76, 196]),
    Rgb([48, 50, 236]),
    Rgb([92, 30, 228]),
    Rgb([136, 20, 176]),
    Rgb([160, 20, 100]),
    Rgb([152, 34, 32]),
    Rgb([120, 60, 0]),
    Rgb([84, 90, 0]),
    Rgb([40, 114, 0]),
    Rgb([8, 124, 0]),
    Rgb([0, 118, 40]),
    Rgb([0, 102, 120]),
    Rgb([0, 0, 0]),
    Rgb([0, 0, 0]),
    Rgb([0, 0, 0]),
    Rgb([236, 238, 236]),
    Rgb([76, 154, 236]),
    Rgb([120, 124, 236]),
    Rgb([176, 98, 236]),
    Rgb([228, 84, 236]),
    Rgb([236, 88, 180]),
    Rgb([236, 106, 100]),
    Rgb([212, 136, 32]),
    Rgb([160, 170, 0]),
    Rgb([116, 196, 0]),
    Rgb([76, 208, 32]),
    Rgb([56, 204, 108]),
    Rgb([56, 180, 204]),
    Rgb([60, 60, 60]),
    Rgb([0, 0, 0]),
    Rgb([0, 0, 0]),
    Rgb([236, 238, 236]),
    Rgb([168, 204, 236]),
    Rgb([188, 188, 236]),
    Rgb([212, 178, 236]),
    Rgb([236, 174, 236]),
    Rgb([236, 174, 212]),
    Rgb([236, 180, 176]),
    Rgb([228, 196, 144]),
    Rgb([204, 210, 120]),
    Rgb([180, 222, 120]),
    Rgb([168, 226, 144]),
    Rgb([152, 226, 180]),
    Rgb([160, 214, 228]),
    Rgb([160, 162, 160]),
    Rgb([0, 0, 0]),
    Rgb([0, 0, 0]),
];
