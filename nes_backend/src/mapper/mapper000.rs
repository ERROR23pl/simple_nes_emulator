use super::*;

#[derive(Default)]
pub struct Mapper000 {
    double_memory: bool,
}

impl Mapper000 {
    pub fn new(double_memory: bool) -> Self {
        Self { double_memory }
    }
}

impl Mapper for Mapper000 {
    fn map_cpu_read(&self, address: u16) -> u32 {
        match address {
            0x0000..0x8000 => unreachable!("This is not meant to be called with these addresses."),
            0x8000.. => (address & if self.double_memory { 0x7FFF } else { 0x3FFF }) as u32,
        }
    }

    fn map_cpu_write(&mut self, _address: u16) -> u32 {
        panic!("Mapper 000 does not have PRG RAM")
    }

    fn map_ppu_read(&self, address: u16) -> u32 {
        match address {
            0x0000..0x2000 => address as u32,
            0x2000.. => unreachable!("This is not meant to be called with these addresses."),
        }
    }

    fn map_ppu_write(&mut self, _address: u16) -> u32 {
        panic!("Mapper 000 does not have CHR RAM.")        
    }
}