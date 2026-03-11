use modular_bitfield::prelude::*;

use std::cell::Cell;

// todo: I'm not sure about all thatk
#[derive(Default)]
pub struct PPUControlRegisters {
    // actual registers
    pub control: PPUControl,
    pub mask: PPUMask,
    pub status: PPUStatus,
    _unused1: u8,
    _unused2: u8,
    pub scroll: u8,
    pub data: u8,

    // helper registers
    pub vram_address: PPULoopy,
    pub tram_address: PPULoopy,
    pub fine_x: u8,
    
    // helper variables
    pub writing_part: WritingAddressPart,
    pub data_buffer: Cell<u8>,
    pub address: Cell<u16>, // actuall 14 or 15 bits?,

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
    sprite_overflow: B1,
    sprite_zero_hit: B1,
    vertical_blank: B1,
}

#[bitfield]
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub struct PPUMask {
    grayscale: B1,
    render_background_left: B1,
    render_sprites_left: B1,
    render_background: B1,
    render_sprites: B1,
    enhance_red: B1,
    enhance_green: B1,
    enhance_blue: B1,
}

#[bitfield]
#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub struct PPUControl {
    nametable_x: B1,
    nametable_y: B1,
    increment_mode: B1,
    pattern_sprite: B1,
    pattern_background: B1,
    sprite_size: B1,
    slave_mode: B1, // unused
    enable_nmi: B1,
}

#[bitfield]
#[repr(u16)]
#[derive(Default, Clone, Copy)]
pub struct PPULoopy {
    coarse_x: B5,
    coarse_y: B5,
    nametable_x: B1,
    nametable_y: B1,
    fine_y: B3,
    unused: B1, 
}

pub const CONTROL: u16 = 0;
pub const MASK: u16 = 1;
pub const STATUS: u16 = 2;
pub const OAM_ADDRESS: u16 = 3;
pub const OAM_DATA: u16 = 4;
pub const SCROLL: u16 = 5;
pub const PPU_ADDRESS: u16 = 6;
pub const PPU_DATA: u16 = 7;

impl PPUControlRegisters {
    // pub fn cpu_read(&mut self, address: u16) -> u8 {
    //     match address {
    //         0 => todo!(),
    //         1 => todo!(),
    //         2 => todo!(),
    //         3 => todo!(),
    //         4 => todo!(),
    //         5 => todo!(),
    //         6 => todo!(),
    //         7 => todo!(),
    //         8.. => unreachable!("This function should only be called with mirroring from either Bus or Cartrige."),
    //     }
    // }
    
    // pub fn cpu_write(&mut self, address: u16, data: u8, bus: &mut Bus) {
    //     match address {
    //         CONTROL => { self.control = data.into() },
    //         MASK => { self.mask = data.into() },
    //         STATUS => panic!("You can't write to the status register."),
    //         3 => todo!(),
    //         4 => todo!(),
    //         5 => todo!(),
    //         PPU_ADDRESS => {
    //             use WritingAddressPart as WAP;
    //             match self.writing_part {
    //                 WAP::High => { self.address = (self.address & 0x00FF) | ((data as u16) << 8) },
    //                 WAP::Low => { self.address = (self.address & 0x00FF) | (data as u16) },
    //             }
    //             self.address_part_switch();
    //         },
    //         PPU_DATA => todo!("Actual PPU writing"),
    //         8.. => unreachable!("This function should only be called with mirroring from either Bus or Cartrige."),
    //     }
    // }
}

// helper functions
impl PPUControlRegisters {
    pub fn address_part_switch(&mut self) {
        self.writing_part = match self.writing_part {
            WritingAddressPart::High => WritingAddressPart::Low,
            WritingAddressPart::Low => WritingAddressPart::High,
        }
    }
}