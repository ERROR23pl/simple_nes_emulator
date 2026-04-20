use modular_bitfield::prelude::*;

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
    pub fn increment(&mut self, value: u16) {
        *self = PPULoopy::from(u16::from(*self).wrapping_add(value));
    }

    pub fn map<R>(&self, function: fn(u16) -> R) -> R {
        function(u16::from(self.clone()))
    }

    pub fn dissolve_u16(&self) -> (u16, u16, u16, u16, u16) {
        (self.fine_y() as u16, self.nametable_y() as u16, self.nametable_x() as u16, self.coarse_y() as u16, self.coarse_x() as u16)
    }
}

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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modular_bitfield_test() {
        assert_eq!(0b0_000_00_00000_10100, u16::from(PPULoopy::new().with_coarse_x(0b10100)));
        assert_eq!(0b0_000_00_10100_00000, u16::from(PPULoopy::new().with_coarse_y(0b10100)));
        assert_eq!(0b0_000_01_00000_00000, u16::from(PPULoopy::new().with_nametable_x(true)));
        assert_eq!(0b0_000_10_00000_00000, u16::from(PPULoopy::new().with_nametable_y(true)));
        assert_eq!(0b0_100_00_00000_00000, u16::from(PPULoopy::new().with_fine_y(0b100)));
        assert_eq!(0b0_100_01_00100_01000, u16::from(PPULoopy::from(0b0_100_01_00100_01000)));
        assert_eq!(0x00FF, u16::from(PPULoopy::from_bytes([0xFF, 0x00])));
        assert_eq!([0xFF, 0x00], PPULoopy::from(0x00FF).into_bytes());
    }
}