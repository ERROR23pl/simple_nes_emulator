use crate::mapper::Mapper;
use crate::file_loading::*;

use crate::mapper::mapper000::Mapper000;


#[derive(Default)]
pub struct Cartridge {
    // these have to be properly sized
    prg_memory: Vec<u8>,
    chr_memory: Vec<u8>,

    mapper: Box<dyn Mapper>, // there are more than 256 mapper afaik
}

#[derive(Debug)]
pub struct UnimplementedMapperError(u8);

impl Cartridge {
    pub fn new(file: &INesFile) -> Result<Self, UnimplementedMapperError> {
        Ok(Cartridge {
            prg_memory: file.prg_rom_data().clone(),
            chr_memory: file.chr_rom_data().clone(),
            mapper: match file.header().mapper_number() {
                0 => Ok(Box::new(Mapper000::default())),
                n => Err(UnimplementedMapperError(n)),
            }?,
        })
    }
}


impl Cartridge {
    pub fn cpu_read(&self, address: u16) -> u8 {
        self.prg_memory[self.mapper.map_cpu_read(address) as usize]
    }
    
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        self.prg_memory[self.mapper.map_cpu_write(address) as usize] = data;
    }
    
    pub fn ppu_read(&self, address: u16) -> u8 {
        self.chr_memory[self.mapper.map_ppu_read(address) as usize]
    }
    
    pub fn ppu_write(&mut self, address: u16, data: u8) {
        self.chr_memory[self.mapper.map_ppu_write(address) as usize] = data;
    }
}