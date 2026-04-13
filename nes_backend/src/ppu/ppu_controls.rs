use modular_bitfield::prelude::*;

use std::cell::Cell;

// todo: I'm not sure about all thatk
#[derive(Default)]
pub struct PPUControlRegisters {
    // actual registers
    pub control: PPUControl, // $2000
    pub mask: PPUMask,       // $2001
    pub status: PPUStatus,   // $2002
    _unused1: u8,            // $2003
    _unused2: u8,            // $2004
    pub scroll: u8,          // $2005
    // todo: understand the addr register
    pub data: u8,            // $2007
    
    // helper registers
    pub vram_address: PPULoopy,
    pub tram_address: PPULoopy,
    pub fine_x: u8,
    
    // helper variables
    pub writing_part: WritingAddressPart,
    pub data_buffer: u8,
    // pub address: u16, // actually 14 or 15 bits?,
}

#[derive(Default)]
pub enum WritingAddressPart {
    #[default]
    High = 0,
    Low = 1,
}

#[bitfield]
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub struct PPUStatus {
    #[skip] __: B5,
    sprite_overflow: bool,
    sprite_zero_hit: bool,
    pub vertical_blank: bool,
}

#[bitfield]
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub struct PPUMask {
    pub grayscale: bool,
    pub render_background_left: bool,
    pub render_sprites_left: bool,
    pub render_background: bool,
    pub render_sprites: bool,
    pub enhance_red: bool,
    pub enhance_green: bool,
    pub enhance_blue: bool,
}

#[bitfield]
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub struct PPUControl {
    pub nametable_x: bool,
    pub nametable_y: bool,
    pub increment_mode: bool,
    pub pattern_sprite: bool,
    pub pattern_background: bool,
    pub sprite_size: bool,
    pub slave_mode: bool, // unused
    pub enable_nmi: bool,
}

// todo: this does not add up to 16!!!!!!!!!!!
#[bitfield]
#[repr(u16)]
#[derive(Default, Clone, Copy)]
pub struct PPULoopy {
    pub coarse_x: B5,
    pub coarse_y: B5,
    pub nametable_x: bool,
    pub nametable_y: bool,
    pub fine_y: B3,
    #[skip] __: B1, 
}

impl PPULoopy {
    pub fn increment(&mut self, value: i32) {
        *self = PPULoopy::from(u16::from(*self) + value as u16);
    }
}

pub const CONTROL: u16 = 0;
pub const MASK: u16 = 1;
pub const STATUS: u16 = 2;
pub const OAM_ADDRESS: u16 = 3;
pub const OAM_DATA: u16 = 4;
pub const SCROLL: u16 = 5;
pub const PPU_ADDRESS: u16 = 6;
pub const PPU_DATA: u16 = 7;

// helper functions
impl PPUControlRegisters {
    pub fn address_part_switch(&mut self) {
        use WritingAddressPart as WP;
        self.writing_part = match self.writing_part {
            WP::High => WP::Low,
            WP::Low => WP::High,
        }
    }
}