pub const KB: usize = 1024;

pub trait BitManipulation {
    fn nth_bit(&self, bit_index: u8) -> u8;
    fn nth_flag(&self, bit_index: u8) -> bool {
        match self.nth_bit(bit_index) {
            0 => false,
            1 => true,
            2.. => unreachable!("GetBits::nth_bit should return only 1 or 0"),
        }
    }

    fn mask(&self, higher_bound: u8, lower_bound: u8) -> Self;
}

impl BitManipulation for u8 {
    fn nth_bit(&self, bit_index: u8) -> u8 {
        // self & (1 << bit_index) masks off the proper bit
        // >> bit_index shifts the index to the first digit
        // so the result is 0 or 1
        (self & (1 << bit_index)) >> bit_index
    }
    
    fn mask(&self, higher_bound: u8, lower_bound: u8) -> Self {
        let length = higher_bound - lower_bound + 1;

        (self >> lower_bound) & (0xFF << length)
    }
}

impl BitManipulation for u16 {
    fn nth_bit(&self, bit_index: u8) -> u8 {
        // self & (1 << bit_index) masks off the proper bit
        // >> bit_index shifts the index to the first digit
        // so the result is 0 or 1
        ((self & (1 << bit_index)) >> bit_index) as u8
    }
    
    fn mask(&self, higher_bound: u8, lower_bound: u8) -> Self {
        let length = higher_bound - lower_bound + 1;

        (self >> lower_bound) & (0xFF << length)
    }
}

#[inline]
pub fn glue_u8s(high_byte: u8, low_byte: u8) -> u16 {
    ((high_byte as u16) << 8) | (low_byte as u16)
}

#[inline]
pub fn split_u16(whole: u16) -> (u8, u8) {
    (get_high_byte(whole), get_low_byte(whole)) 
}


#[inline]
pub fn get_high_byte(number: u16) -> u8 {
    ((number & 0xFF00) >> 8) as u8
}

#[inline]
pub fn get_low_byte(number: u16) -> u8 {
    (number & 0x00FF) as u8
}