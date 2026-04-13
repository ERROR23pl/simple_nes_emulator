pub mod ppu_controls;
mod ppu_clock;

use log::{error, warn};

use crate::bus::Bus;
use crate::ppu::ppu_controls::{PPUControl, PPUMask};
use crate::rendering::{NesColor, PAL_COLOR, PatternTable, PixelBuffer};
use crate::utils::Shared;

pub struct PPU<P: PixelBuffer> {
    // pattern memory is located on the cardridge
    // conceptually it can be accessed with addresses $0000..=$1FFF
    // meaning that logically there's 8KB of memory there.
    // mappers can of course change this behaviour, but as far as
    // PPU is concerned, there are 8KB of pattern memory
    // These are split in two 4KB chunks sometimes called "left" and "right"
    // Pattern memory is usually ROM, though it can be RAM
    bus: Shared<Bus>,

    // helper fields needed for emulation
    scanline: i16,
    cycle: u16,
    frame_complete: bool,
    pub nmi: bool,

    // helper for rendering
    bg_next_tile_id: u8,
	bg_next_tile_attrib: u8,
	bg_next_tile_lsb: u8,
	bg_next_tile_msb: u8,
	bg_shifter_pattern_lo: u16,
	bg_shifter_pattern_hi: u16,
	bg_shifter_attrib_lo: u16,
	bg_shifter_attrib_hi: u16,

    // rendering tool to render emulate sending a signal to TV.
    // It's generic to allow multiple implementations. 
    screen: P,
    pattern_table_screen: P,
}

impl<P: PixelBuffer> PPU<P> {
    pub fn new(screen_buffer: P, pattern_table_screen: P, bus: Shared<Bus>) -> Self {
        PPU {
            bus,

            scanline: 0,
            cycle:0,
            frame_complete: false,
            nmi: false,
            
            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,

            screen: screen_buffer,
            pattern_table_screen
        }
    }
    
    pub fn ppu_read(&self, address: u16) -> u8 {
        self.bus.borrow_mut().ppu_read(address)
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        self.bus.borrow_mut().ppu_write(address, data)
    }
    
    pub fn get_color_value_from_pallet_ram(&self, pallette_id: u8, pixel_id: u8) -> u8 {
        self.ppu_read(0x3F00 + (pallette_id as u16 * 4) + pixel_id as u16)
    }
}

// debug functions
impl<P: PixelBuffer> PPU<P> {
    fn set_pixel_pattern_table(&mut self, pattern_table: PatternTable, x: usize, y: usize, color: NesColor) {
        let pattern_shift = (pattern_table as usize) * (256 / 2);
        self.pattern_table_screen.set(x + pattern_shift + y * 256, color);
    }

    pub fn render_debug_pattern_table(&mut self, pattern_table_side: PatternTable, pallete_id: u8) {
        // locate the tile in the pattern table
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                const SINGLE_PATTERN_TABLE_SIZE: u16 = 0x1000;
                let offset = (pattern_table_side as u16) * SINGLE_PATTERN_TABLE_SIZE + tile_y * 256 + tile_x * 16;

                // each tile is 8x8 pixels. Rows are stored one after another.
                for row in 0..8 {
                    // each row is 2 bytes stored one after another
                    let mut tile_lsb = self.ppu_read(offset + row + 0); 
                    let mut tile_msb = self.ppu_read(offset + row + 8); 

                    for col in 0.. 8 {
                        // pixels are encoded by 2 bits
                        let pixel: u8 = (tile_lsb & 0x01) + (tile_msb & 0x01);
                        tile_lsb >>= 1;
                        tile_msb >>= 1;

                        self.set_pixel_pattern_table(
                            pattern_table_side,
                            (tile_x * 8 + (7 - col)) as usize, // `7 - col` since we read it in reverse order
                            (tile_y * 8 + row) as usize,
                            self.get_color_value_from_pallet_ram(pallete_id, pixel)
                        );
                    }
                }
            }
        }
    }
}