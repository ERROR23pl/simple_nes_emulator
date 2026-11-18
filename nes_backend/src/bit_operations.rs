pub const KB: usize = 1024;

pub trait BitManipulation {
    fn nth_flag(self, bit_index: u8) -> bool;
    fn nth_bit<T: From<bool>>(self, bit_index: u8) -> T where Self: Sized{
        T::from(self.nth_flag(bit_index))
    }
}

macro_rules! impl_bit_maniuplation {
    ($type_: tt) => {
        impl BitManipulation for $type_ {
            fn nth_flag(self, bit_index: u8) -> bool {
                let mask = (1 as $type_) << bit_index;

                (self & mask) != 0
            }
        }       
    };
}

impl_bit_maniuplation!(u8);
impl_bit_maniuplation!(u16);



#[inline]
pub fn glue_u8s(high_byte: u8, low_byte: u8) -> u16 {
    ((high_byte as u16) << 8) | (low_byte as u16)
}

#[inline]
/// returns tuple `(high_byte, low_byte)`
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

/// performs a "mirror" flip of the byte so that
/// every bit on a position `x` ends up on position `7-x`.
pub fn flip_byte(b: u8) -> u8 {
    let b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
    let b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
    let b = (b & 0xAA) >> 1 | (b & 0x55) << 1;

    b
}


pub fn mask(number: u8, upper: u8, lower: u8) -> u8 {
    let number_of_ones = (upper - lower) + 1;
    let temp = u8::MAX >> (8 - number_of_ones);
    let mask = temp << lower;

    let masked_number = number & mask;

    masked_number >> lower
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mask_test() {
        assert_eq!(mask(0b0000_0001, 0, 0), 0b1);
        assert_eq!(mask(0b0000_1000, 3, 3), 0b1);
        assert_eq!(mask(0b0000_1000, 3, 2), 0b10);
        assert_eq!(mask(0b0000_1010, 3, 1), 0b101);
        assert_eq!(mask(0b0000_1010, 3, 0), 0b1010);
        assert_eq!(mask(0b0101_1010, 6, 3), 0b1011);
    }
}