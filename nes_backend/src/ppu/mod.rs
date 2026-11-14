mod ppu_controls;
mod ppu_clock;
mod oam;
mod debug;

pub use oam::*;
pub use ppu_controls::*;
use crate::cartridge::{Cartridge, Mirroring};
use crate::ppu::ppu_controls::PPUControlRegisters;
use crate::rendering::{NesColor, PatternTable, PixelBuffer};

use itertools::iproduct;
use log::error;

pub struct PPU {
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

    // Object Attribute Memory (OAM, used for sprites AKA foreground graphics)
    pub oam: OAM,
    pub oam_address: u8,
    sprite_scanline: [OamEntry; 8],
    sprite_count: u8,
    sprite_shifter_lo: [u8; 8],
    sprite_shifter_hi: [u8; 8],
    sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,
}

impl PPU {
    pub fn new() -> Self {
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

            oam: OAM::default(),
            oam_address: 0,
            sprite_scanline: [OamEntry::default(); 8],
            sprite_count: 0,
            sprite_shifter_lo: [0; 8],
            sprite_shifter_hi: [0; 8],
            sprite_zero_hit_possible: false,
            sprite_zero_being_rendered: false,
        }
    }
    
    pub fn read(&self, address: u16, cartridge: &Cartridge) -> u8 {
        match address {
            // pattern memory is located on the cardridge
            // conceptually it can be accessed with addresses $0000..=$1FFF
            // meaning that logically there's 8KB of memory there.
            // mappers can of course change this behaviour, but as far as
            // PPU is concerned, there are 8KB of pattern memory
            // These are split in two 4KB chunks sometimes called "left" and "right"
            // Pattern memory is usually ROM, though it can be RAM
            0x0000..0x2000 => { cartridge.ppu_read(address) },

            // todo: nametables can be also mapped by a cartridge
            0x2000..0x3000 => match cartridge.mirroring() {
                // the mirroring is done by a brute force method even though there are more clever solutions.
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

            0x3000..0x3F00 => {
                error!("memory 0x3000..0x3F00 on PPU bus is unused, but a read on {:04X} was attempted. Dummy 0 returned.", address);
                0
            },
            
            // pallette memory
            0x3F00..0x4000 => {
                // todo: I don't know if this way of mirroring works lol, I should check; 
                let masked_address = address & 0x001F;
                let mirrored_address = if masked_address % 4 == 0 { 0 } else { masked_address };
                // let mirrored_address = if (address & 0x03) == 0 { 0 } else { address & 0x1F };
                self.palette[mirrored_address as usize] & (if self.ppu_controls.mask.grayscale() { 0x30 } else { 0x3F })

                // let mirrored_address = match masked_address {
                //     0x0010 => 0x0004,
                //     0x0014 => 0x0004,
                //     0x0018 => 0x0008,
                //     0x001C => 0x0012,
                //     _ => masked_address,
                // };
                // self.palette[mirrored_address as usize]

                // Jarvis Way
                // let mut addr = address & 0x001F;
                // if addr == 0x0010 { addr = 0x0000 };
                // if addr == 0x0014 { addr = 0x0004 };
                // if addr == 0x0018 { addr = 0x0008 };
                // if addr == 0x001C { addr = 0x000C };
                // self.palette[addr as usize] & (if self.ppu_controls.mask.grayscale() { 0x30 } else { 0x3F })
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

            0x3000..0x3F00 => {
                error!("memory 0x3000..0x3F00 on PPU bus is unused, write of value {:02X} in address {:04X} was attempted.", data, address);
            },

            // pallette memory
            0x3F00..0x4000 => {
                // todo: I don't know if this way of mirroring works lol, I should check; 
                
                // My way
                let masked_address = address & 0x001F;
                let mirrored_address = if masked_address % 4 == 0 { 0 } else { masked_address };
                // let mirrored_address = if address & 0x03 == 0 { 0 } else { address & 0x1F };
                self.palette[mirrored_address as usize] = data; 

                // Jarvis way
                // let mut addr = address & 0x001F;
                // if addr == 0x0010 { addr = 0x0000 };
                // if addr == 0x0014 { addr = 0x0004 };
                // if addr == 0x0018 { addr = 0x0008 };
                // if addr == 0x001C { addr = 0x000C };
                // self.palette[addr as usize] = data;
            },

            0x4000.. => unreachable!("PPU bus only goes from has address range 0x0000..0x4000"),
        }
    }

    // todo: don't know if it works, should test
    pub fn get_color_value_from_pallet_ram(&self, pallette_id: u8, pixel_id: u8) -> NesColor {
        let initial_address = (pallette_id as usize * 4) + pixel_id as usize;

        // My way
        // let mirrored_address = initial_address & 0x001F;
        // let addr = if pixel_id == 0 { 0 } else { mirrored_address }; 

        // let mirrored_address = if initial_address & 0x03 == 0 { 0 } else { initial_address & 0x1F };
        // let mirrored_address = match initial_address {
        //     0x0010 => 0x0004,
        //     0x0014 => 0x0004,
        //     0x0018 => 0x0008,
        //     0x001C => 0x0012,
        //     _ => initial_address,
        // };
        // let mirrored_address = if pixel_id == 0 { 0 } else { initial_address }; 
        // self.palette[mirrored_address as usize]

        // Jarvis way
        let mut addr = initial_address & 0x001F;
        if addr == 0x0010 { addr = 0x0000 };
        if addr == 0x0014 { addr = 0x0004 };
        if addr == 0x0018 { addr = 0x0008 };
        if addr == 0x001C { addr = 0x000C };
        self.palette[addr as usize] & (if self.ppu_controls.mask.grayscale() { 0x30 } else { 0x3F })
    }
}
