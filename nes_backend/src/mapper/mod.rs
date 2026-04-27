pub mod mapper000;

use crate::mapper::mapper000::Mapper000;


pub trait Mapper {
    fn map_cpu_read(&self, address: u16) -> u32;
    fn map_cpu_write(&mut self, address: u16) -> u32;
    fn map_ppu_read(&self, address: u16) -> u32;
    fn map_ppu_write(&mut self, address: u16) -> u32;
}

impl Default for Box<dyn Mapper> {
    fn default() -> Self {
        Box::new(Mapper000::default())
    }
}