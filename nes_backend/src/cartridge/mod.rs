use thiserror::Error;

// crate imports
use crate::mapper::Mapper;
use crate::file_loading::{self, *};

use crate::mapper::mapper000::Mapper000;


#[derive(derive_getters::Getters)]
pub struct Cartridge {
    pub prg_memory: Vec<u8>,
    chr_memory: Vec<u8>,
    mirroring: Mirroring,
    mapper: Box<dyn Mapper>,
}

#[derive(Default)]
pub enum Mirroring {
    #[default]
    Horizontal,
    Vertical,
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            prg_memory: vec![0; PRG_BANK_SIZE],
            chr_memory: vec![0; CHR_BANK_SIZE],
            mirroring: Mirroring::Horizontal,
            mapper: Default::default(),
        }
    }
}

impl TryFrom<&INesFile> for Cartridge {
    type Error = UnimplementedMapperError;

    fn try_from(file: &INesFile) -> Result<Self, Self::Error> {
        Ok(Cartridge {
            prg_memory: file.prg_rom_data().clone(),
            chr_memory: file.chr_rom_data().clone(),
            mirroring: file.header().nametable_arrangement().into(),
            mapper: match file.header().mapper_number() {
                0 => Ok(Box::new(Mapper000::new(file.prg_rom_data().len() > file_loading::PRG_BANK_SIZE))),
                n => Err(UnimplementedMapperError(n)),
            }?,
        })
    }
}

#[derive(Debug, Error)]
#[error("mapper #{0} has not yet been implemented")]
pub struct UnimplementedMapperError(u8);


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


// * debugging
impl Cartridge {
    pub fn example() -> Self {
        let mut data: Vec<u8> = vec![
            0xA2, 0x00, 0xBD, 0x00, 0x02, 0x9D, 0x00, 0x03,
            0xE8, 0xBD, 0x00, 0x02, 0x9D, 0x00, 0x03, 0xE8,
            0xBD, 0x00, 0x02, 0x9D, 0x00, 0x03, 0xE8, 0xBD,
            0x00, 0x02, 0x9D, 0x00, 0x03, 0xE8, 0xBD, 0x00,
            0x02, 0x9D, 0x00, 0x03, 0xE8, 0xBD, 0x00, 0x02,
            0x9D, 0x00, 0x03, 0xE8, 0x00,
        ];
        data.extend(vec![0u8; (1 << 14) - data.len()]);
        Cartridge {
            prg_memory: data,
            chr_memory: vec![0; 1 << 13],
            mapper: Box::new(Mapper000::new(false)),
            ..Default::default()
        }
    }
}

impl From<&NametableArrangement> for Mirroring {
    fn from(value: &NametableArrangement) -> Self {
        use NametableArrangement as NA;
        match value {
            NA::Horizontal => Mirroring::Horizontal,
            NA::Vertical => Mirroring::Vertical,
        }
    }
}