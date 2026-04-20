pub const KB: usize = 1024;

pub trait GetBits {
    fn nth_bit<const INDEX: u8>(&self) -> u8;
    fn nth_flag<const INDEX: u8>(&self) -> bool {
        match self.nth_bit::<INDEX>() {
            0 => false,
            1 => true,
            2.. => unreachable!("GetBits::nth_bit should return only 1 or 0"),
        }
    }
    // todo: bits
}

impl GetBits for u8 {
    #[inline]
    fn nth_bit<const INDEX: u8>(&self) -> u8 {
        (self & (1 << INDEX)) >> INDEX
    }
}

pub trait GetBitsGeneric {
    fn nth_bit_gen<const N: u8>(&self) -> u8;
}

impl GetBitsGeneric for u8 {
    #[inline]
    fn nth_bit_gen<const N: u8>(&self) -> u8 {
        (self & (1 << N)) >> N
    }
}

#[inline]
pub fn glue_u8s(high_byte: u8, low_byte: u8) -> u16 {
    ((high_byte as u16) << 8) | (low_byte as u16)
}

#[inline]
pub fn split_u16(whole: u16) -> (u8, u8) {
    let low_byte = (whole & 0x00FF) as u8;
    let high_byte = get_high_byte(whole);
    
    (high_byte, low_byte) 
}

#[inline]
pub fn get_high_byte(number: u16) -> u8 {
    ((number & 0xFF00) >> 8) as u8
}
