use modular_bitfield::bitfield;

use crate::rendering::*;

#[derive(Clone, Copy)]
#[bitfield]
#[repr(u32)]
pub struct Pixel {
    blue: u8,
    green: u8,
    red: u8,
    #[skip] __: modular_bitfield::prelude::B8,
}

impl Pixel {
    const BLACK: Pixel = Pixel::from_bytes([0u8; 4]);
    const WHITE: Pixel = Pixel::from_bytes([255u8; 4]);
}

impl Pixel {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }

    pub const PAL_COLOR: [u32; 0x40] = [
        Pixel::from_rgb(84, 84, 84),
        Pixel::from_rgb(0, 30, 116),
        Pixel::from_rgb(8, 16, 144),
        Pixel::from_rgb(48, 0, 136),
        Pixel::from_rgb(68, 0, 100),
        Pixel::from_rgb(92, 0, 48),
        Pixel::from_rgb(84, 4, 0),
        Pixel::from_rgb(60, 24, 0),
        Pixel::from_rgb(32, 42, 0),
        Pixel::from_rgb(8, 58, 0),
        Pixel::from_rgb(0, 64, 0),
        Pixel::from_rgb(0, 60, 0),
        Pixel::from_rgb(0, 50, 60),
        Pixel::from_rgb(0, 0, 0),
        Pixel::from_rgb(0, 0, 0),
        Pixel::from_rgb(0, 0, 0),

        Pixel::from_rgb(152, 150, 152),
        Pixel::from_rgb(8, 76, 196),
        Pixel::from_rgb(48, 50, 236),
        Pixel::from_rgb(92, 30, 228),
        Pixel::from_rgb(136, 20, 176),
        Pixel::from_rgb(160, 20, 100),
        Pixel::from_rgb(152, 34, 32),
        Pixel::from_rgb(120, 60, 0),
        Pixel::from_rgb(84, 90, 0),
        Pixel::from_rgb(40, 114, 0),
        Pixel::from_rgb(8, 124, 0),
        Pixel::from_rgb(0, 118, 40),
        Pixel::from_rgb(0, 102, 120),
        Pixel::from_rgb(0, 0, 0),
        Pixel::from_rgb(0, 0, 0),
        Pixel::from_rgb(0, 0, 0),

        Pixel::from_rgb(236, 238, 236),
        Pixel::from_rgb(76, 154, 236),
        Pixel::from_rgb(120, 124, 236),
        Pixel::from_rgb(176, 98, 236),
        Pixel::from_rgb(228, 84, 236),
        Pixel::from_rgb(236, 88, 180),
        Pixel::from_rgb(236, 106, 100),
        Pixel::from_rgb(212, 136, 32),
        Pixel::from_rgb(160, 170, 0),
        Pixel::from_rgb(116, 196, 0),
        Pixel::from_rgb(76, 208, 32),
        Pixel::from_rgb(56, 204, 108),
        Pixel::from_rgb(56, 180, 204),
        Pixel::from_rgb(60, 60, 60),
        Pixel::from_rgb(0, 0, 0),
        Pixel::from_rgb(0, 0, 0),

        Pixel::from_rgb(236, 238, 236),
        Pixel::from_rgb(168, 204, 236),
        Pixel::from_rgb(188, 188, 236),
        Pixel::from_rgb(212, 178, 236),
        Pixel::from_rgb(236, 174, 236),
        Pixel::from_rgb(236, 174, 212),
        Pixel::from_rgb(236, 180, 176),
        Pixel::from_rgb(228, 196, 144),
        Pixel::from_rgb(204, 210, 120),
        Pixel::from_rgb(180, 222, 120),
        Pixel::from_rgb(168, 226, 144),
        Pixel::from_rgb(152, 226, 180),
        Pixel::from_rgb(160, 214, 228),
        Pixel::from_rgb(160, 162, 160),
        Pixel::from_rgb(0, 0, 0),
        Pixel::from_rgb(0, 0, 0)
    ];
}

pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    pub const fn new(width: usize, height: usize) -> Self {
        Dimensions { width, height }
    }

    pub const fn num_of_pixels(&self) -> usize {
        self.height * self.width
    }
}

pub const EMULATOR_SCREEN_SIZE: Dimensions = Dimensions::new(256, 240);
pub const PATTERN_TABLES_SIZE: Dimensions = Dimensions::new(256, 128);

pub struct MiniFBScreenBuffer {
    pub main_screen: [NesColor; EMULATOR_SCREEN_SIZE.num_of_pixels()],
    pub pattern_screen: [NesColor; PATTERN_TABLES_SIZE.num_of_pixels()],
}


pub fn render(buffer: &[NesColor]) -> [Pixel; EMULATOR_SCREEN_SIZE.num_of_pixels()] {
    let mut new_buffer: [Pixel; EMULATOR_SCREEN_SIZE.num_of_pixels()] = [0u32.into(); _];

    for (i, n) in buffer.iter().enumerate() {
        new_buffer[i] = Pixel::PAL_COLOR[*n as usize].into();
    }

    new_buffer
}

impl PixelBuffer for MiniFBScreenBuffer {
    fn get_pixel(&self, x: usize, y: usize) -> NesColor {
        self.main_screen[x + y * EMULATOR_SCREEN_SIZE.width]
    }
    
    fn set_pixel(&mut self, x: usize, y: usize, color: NesColor) {
        self.main_screen[x + y * EMULATOR_SCREEN_SIZE.width] = color;
    }
    
    fn get(&self, index: usize) -> NesColor {
        self.main_screen[index]
    }
    
    fn set(&mut self, index: usize, color: NesColor) {
        self.main_screen[index] = color;
    }
    
    fn get_pixel_pattern_table(&self, pattern_table: PatternTable, x: usize, y: usize) -> NesColor {
        let pattern_shift = pattern_table as usize * EMULATOR_SCREEN_SIZE.width / 2;
        self.pattern_screen[x + pattern_shift + y * EMULATOR_SCREEN_SIZE.width]
    }
    
    fn set_pixel_pattern_table(&mut self, pattern_table: PatternTable, x: usize, y: usize, color: NesColor) {
        let pattern_shift = pattern_table as usize * EMULATOR_SCREEN_SIZE.width / 2;
        self.pattern_screen[x + pattern_shift + y * EMULATOR_SCREEN_SIZE.width] = color;
    }

    fn render_frame(&mut self) {
        todo!()
    }
}
