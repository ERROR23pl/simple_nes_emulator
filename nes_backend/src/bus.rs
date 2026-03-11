
use crate::{bit_operations::KB, cartridge::Cartridge, ppu::ppu_controls::*};

const CPU_RAM_SIZE: usize = 2 * KB;

pub struct Bus {
    cpu_ram: [u8; CPU_RAM_SIZE],
    ppu_controls: PPUControlRegisters,
    
    cartridge: Cartridge,
    
    name_table: [[u8; 1024]; 2], // NES stores two 1KB name tables (vram)
    palette: [u8; 32],
}

macro_rules! RAM_RANGE { () => { 0x0000..0x2000 }; }
macro_rules! PPU_RANGE { () => { 0x2000..0x4000 }; }

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cpu_ram: [0; CPU_RAM_SIZE],
            cartridge: cartridge,
            ppu_controls: PPUControlRegisters::default(),
            
            name_table: [[0; 1024]; 2],
            palette: [0; 32],
        }
    }

    pub fn get_cartridge(&self) -> &Cartridge {
        &self.cartridge
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            RAM_RANGE!() => { self.cpu_ram[(address & 0x7FFF) as usize] = data } // & 0x7FFF implements mirroring
            PPU_RANGE!() => { self.cpu_to_ppu_write(address & 0x0007, data) } // 0x0007 implements mirroring
            _ => todo!(),
        }
    }

    pub fn cpu_read(&self, address: u16, read_only: bool /* ? What is this flag for? */) -> u8 {
        match address {
            RAM_RANGE!() => self.cpu_ram[(address & 0x7FFF) as usize],
            PPU_RANGE!() => self.cpu_from_ppu_read(address & 0x0007),
            _ => todo!(), 
        }
    } 

    pub fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0x0000..0x2000 => {
                self.get_cartridge().ppu_read(address)
            },
            0x2000..0x3F00 => todo!("reading from pattern memory on the cartridge"),
            0x3F00..0x4000 => {
                let masked_address = address & 0x001F;
                // todo: make sure this works
                let mirrored_address = match address {
                    0x0010 => 0x0000,
                    0x0014 => 0x0004,
                    0x0018 => 0x0008,
                    0x001C => 0x000C,
                    _ => unreachable!(),
                };

                self.palette[mirrored_address as usize]
            },
            0x4000.. => unreachable!("PPU bus only goes from has address range 0x0000..0x4000"),
        }
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        todo!()
    }

    // ! This will require either a change from &self to &mut self or celling someting
    pub fn cpu_from_ppu_read(&self, address: u16) -> u8 {
        match address {
            0 => todo!(),
            1 => todo!(),
            2 => todo!(),
            3 => todo!(),
            4 => todo!(),
            5 => todo!(),
            PPU_ADDRESS => panic!("You can't read from address register"),
            PPU_DATA => {
                let mut retrieved_data = self.ppu_controls.data_buffer.replace(
                    self.ppu_read(self.ppu_controls.vram_address.into())
                );

                if u16::from(self.ppu_controls.vram_address) >= 0x3F00 {
                    retrieved_data = self.ppu_controls.data_buffer.get();
                }

                todo!()
            },
            8.. => unreachable!("This function should only be called with mirroring from either Bus or Cartrige."),
        }
    }
    
    pub fn cpu_to_ppu_write(&mut self, address: u16, data: u8) {
        match address {
            CONTROL => { self.ppu_controls.control = data.into() },
            MASK => { self.ppu_controls.mask = data.into() },
            STATUS => panic!("You can't write to the status register."),
            3 => todo!(),
            4 => todo!(),
            5 => todo!(),
            PPU_ADDRESS => {
                use WritingAddressPart as WAP;
                let offset = match self.ppu_controls.writing_part { WAP::High => 8, WAP::Low => 0 };
                self.ppu_controls.address.set(
                    (self.ppu_controls.address.get() & 0x00FF) | 
                    ((data as u16) << offset)
                );                

                self.ppu_controls.address_part_switch();
            },
            PPU_DATA => todo!("Actual PPU writing"),
            8.. => unreachable!("This function should only be called with mirroring from either Bus or Cartrige."),
        }
    }
}