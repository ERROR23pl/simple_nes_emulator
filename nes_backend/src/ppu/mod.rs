pub mod ppu_controls;
mod ppu_clock;


use crate::cartridge::{self, Cartridge, Mirroring};
use crate::ppu::ppu_controls::PPUControlRegisters;
use crate::rendering::{NesColor, PAL_COLOR, PatternTable, PixelBuffer};

use itertools::iproduct;

pub struct PPU<P: PixelBuffer> {
    // pattern memory is located on the cardridge
    // conceptually it can be accessed with addresses $0000..=$1FFF
    // meaning that logically there's 8KB of memory there.
    // mappers can of course change this behaviour, but as far as
    // PPU is concerned, there are 8KB of pattern memory
    // These are split in two 4KB chunks sometimes called "left" and "right"
    // Pattern memory is usually ROM, though it can be RAM
    name_table: [[u8; 1024]; 2], // NES stores two 1KB name tables (vram)
    palette: [u8; 32],
    pub ppu_controls: PPUControlRegisters,
    
    // helper fields needed for emulation
    scanline: i16,
    cycle: u16,
    pub frame_complete: bool,
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
    pub screen: P,
    pub pattern_table_screen: P,
}

impl<P: PixelBuffer> PPU<P> {
    pub fn new(screen_buffer: P, pattern_table_screen: P) -> Self {
        PPU {
            name_table: [[0; 1024]; 2],
            palette: [0; 32],
            ppu_controls: PPUControlRegisters::default(),
            
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
    
    pub fn read(&self, address: u16, cartridge: &Cartridge) -> u8 {
        match address {
            // pattern memory
            0x0000..0x2000 => { cartridge.ppu_read(address) },

            // name table
            0x2000..0x3000 => match cartridge.mirroring() {
                // the mirroring is done by a brute force method even thoug there are more clever solutions.
                Mirroring::Horizontal => match address & 0x0FFF {
                    0x0000..0x0400 | 0x0800..0x0C00 => self.name_table[0][(address & 0x03FF) as usize],
                    0x0400..0x0800 | 0x0C00..0x1000 => self.name_table[1][(address & 0x03FF) as usize],
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
                Mirroring::Vertical => match address & 0x0FFF {
                    0x0000..0x0800 => self.name_table[0][(address & 0x03FF) as usize],
                    0x0800..0x1000 => self.name_table[1][(address & 0x03FF) as usize],
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
            },

            0x3000..0x3F00 => { panic!("unused.") },
            
            // pallette memory
            0x3F00..0x4000 => {
                // todo: I don't know if this way of mirroring works lol, I should check; 
                // let masked_address = address & 0x001F;
                // let mirrored_address = if masked_address % 4 == 0 { 0 } else { masked_address };
                let mirrored_address = if address & 0x03 == 0 { 0 } else { address & 0x1F };

                // let mirrored_address = match masked_address {
                //     0x0010 => 0x0004,
                //     0x0014 => 0x0004,
                //     0x0018 => 0x0008,
                //     0x001C => 0x0012,
                //     _ => masked_address,
                // };
                self.palette[mirrored_address as usize]
            },

            0x4000.. => unreachable!("PPU bus only goes from has address range 0x0000..0x4000"),
        }
    }

    pub fn write(&mut self, address: u16, data: u8, cartridge: &mut Cartridge) {
        match address {
            // pattern memory
            0x0000..0x2000 => { cartridge.ppu_write(address, data); },
            
            // nametables
            0x2000..0x3000 => match cartridge.mirroring() {
                // the mirroring is done by a brute force method even thoug there are more clever solutions.
                Mirroring::Horizontal => match address & 0x0FFF {
                    0x0000..0x0400 | 0x0800..0x0C00 => { self.name_table[0][(address & 0x03FF) as usize] = data },
                    0x0400..0x0800 | 0x0C00..0x1000 => { self.name_table[1][(address & 0x03FF) as usize] = data },
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
                Mirroring::Vertical => match address & 0x0FFF {
                    0x0000..0x0800 => { self.name_table[0][(address & 0x03FF) as usize] = data },
                    0x0800..0x1000 => { self.name_table[1][(address & 0x03FF) as usize] = data },
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
            },

            0x3000..0x3F00 => { panic!("unused.") },

            // pallette memory
            0x3F00..0x4000 => {
                // todo: I don't know if this way of mirroring works lol, I should check; 
                // let masked_address = address & 0x001F;
                // let mirrored_address = if masked_address % 4 == 0 { 0 } else { masked_address };
                let mirrored_address = if address & 0x03 == 0 { 0 } else { address & 0x1F };
                // let mirrored_address = match masked_address {
                //     0x0010 => 0x0004,
                //     0x0014 => 0x0004,
                //     0x0018 => 0x0008,
                //     0x001C => 0x0012,
                //     _ => masked_address,
                // };
                self.palette[mirrored_address as usize] = data; 
            },

            0x4000.. => unreachable!("PPU bus only goes from has address range 0x0000..0x4000"),
        }
    }

    // todo: don't know if it works, should test
    pub fn get_color_value_from_pallet_ram(&self, pallette_id: u8, pixel_id: u8) -> NesColor {
        let initial_address = (pallette_id as usize * 4) + pixel_id as usize;
        // let initial_address = initial_address & 0x001F;
        // let mirrored_address = if pixel_id == 0 { 0 } else { initial_address }; 

        let mirrored_address = if initial_address & 0x03 == 0 { 0 } else { initial_address & 0x1F };
        // let mirrored_address = match initial_address {
        //     0x0010 => 0x0004,
        //     0x0014 => 0x0004,
        //     0x0018 => 0x0008,
        //     0x001C => 0x0012,
        //     _ => initial_address,
        // };
        // let mirrored_address = if pixel_id == 0 { 0 } else { initial_address }; 
        self.palette[mirrored_address as usize]
    }
}

// debug functions
impl<P: PixelBuffer> PPU<P> {
    fn set_pixel_pattern_table(&mut self, pattern_table: PatternTable, x: usize, y: usize, color: NesColor) {
        let pattern_shift = (pattern_table as usize) * (256 / 2);
        self.pattern_table_screen.set(x + pattern_shift + y * 256, color);
    }

    pub fn render_debug_pattern_table(&mut self, pattern_table_side: PatternTable, pallete_id: u8, cartridge: &Cartridge) {
        // locate the tile in the pattern table
        const SINGLE_PATTERN_TABLE_SIZE: u16 = 0x1000;
        for (tile_y, tile_x) in iproduct!(0..16, 0..16) {
            let offset = (pattern_table_side as u16) * SINGLE_PATTERN_TABLE_SIZE + tile_y * 256 + tile_x * 16;

            // each tile is 8x8 pixels. Rows are stored one after another.
            for row in 0..8 {
                // each row is 2 bytes stored one after another
                let mut tile_lsb = self.read(offset + row + 0, cartridge); 
                let mut tile_msb = self.read(offset + row + 8, cartridge); 

                for col in 0..8 {
                    // pixels are encoded by 2 bits
                    let pixel: u8 = (tile_lsb & 0x01) | ((tile_msb & 0x01) << 1);
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