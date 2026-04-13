
use log::{error, warn};

use crate::{bit_operations::KB, cartridge::{Cartridge, Mirroring}, ppu::{self, ppu_controls::*}};

const CPU_RAM_SIZE: usize = 2 * KB;

#[derive(derive_getters::Getters)]
pub struct Bus {
    cpu_ram: [u8; CPU_RAM_SIZE],
    ppu_controls: PPUControlRegisters,
    
    cartridge: Cartridge,
    
    name_table: [[u8; 1024]; 2], // NES stores two 1KB name tables (vram)
    palette: [u8; 32],
}

macro_rules! RAM_RANGE { () => { 0x0000..0x2000 }; }
macro_rules! PPU_RANGE { () => { 0x2000..0x4000 }; }
macro_rules! PROGRAM_ROM_RANGE { () => { 0x4020..=0xFFFF }; }

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

    // called by CPU to write to the bus
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            RAM_RANGE!() => { self.cpu_ram[(address & 0x7FFF) as usize] = data }, // & 0x7FFF implements mirroring
            PPU_RANGE!() => { self.cpu_write_to_ppu(address & 0x0007, data) }, // 0x0007 implements mirroring
            PROGRAM_ROM_RANGE!() => { self.cartridge.cpu_write(address, data); } // todo: is this correct?
            _ => { warn!("attempted to write to memory that wasn't yet implemented. Not doing anything.") },
        }
    }
    
    // called by PPU to read from the bus
    pub fn cpu_read(&mut self, address: u16, read_only: bool) -> u8 {
        match address {
            RAM_RANGE!() => self.cpu_ram[(address & 0x7FFF) as usize],
            PPU_RANGE!() => self.cpu_read_from_ppu(address & 0x0007),
            PROGRAM_ROM_RANGE!() => self.cartridge.cpu_read(address), // todo: is this correct?
            _ => { warn!("Attempted to read from memory that wasn't yet implemented. Returning 0."); 0 },
        }
    } 

    pub fn ppu_read(&self, address: u16) -> u8 {
        match address {
            // pattern memory
            0x0000..0x2000 => {
                self.get_cartridge().ppu_read(address)
            },

            // name table
            0x2000..0x3F00 => match self.cartridge().mirroring() {
                // the mirroring is done by a brute force method even thoug there are more clever solutions.
                Mirroring::Horizontal => match address & 0x0FFF {
                    0x0000..0x0400 => self.name_table[0][(address & 0x03FF) as usize],
                    0x0400..0x0800 => self.name_table[1][(address & 0x03FF) as usize],
                    0x0800..0x0C00 => self.name_table[0][(address & 0x03FF) as usize],
                    0x0C00..0x1000 => self.name_table[1][(address & 0x03FF) as usize],
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
                Mirroring::Vertical => match address & 0x0FFF {
                    0x0000..0x0400 => self.name_table[0][(address & 0x03FF) as usize],
                    0x0400..0x0800 => self.name_table[0][(address & 0x03FF) as usize],
                    0x0800..0x0C00 => self.name_table[1][(address & 0x03FF) as usize],
                    0x0C00..0x1000 => self.name_table[1][(address & 0x03FF) as usize],
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
            },

            // pallette memory
            0x3F00..0x4000 => {
                let masked_address = address & 0x001F;
                // 1_1111
                // todo: this doesn't fucking work
                let mirrored_address = masked_address % 32;
                // let mirrored_address = match masked_address {
                //     0x0010 => 0x0000,
                //     0x0014 => 0x0004,
                //     0x0018 => 0x0008,
                //     0x001C => 0x000C,
                //     _ => unreachable!(),
                // };

                self.palette[mirrored_address as usize]
            },
            0x4000.. => unreachable!("PPU bus only goes from has address range 0x0000..0x4000"),
        }
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        match address {
            // pattern memory
            0x0000..0x2000 => {
                self.cartridge.ppu_write(address, data);
            },
            
            // name table
            0x2000..0x3F00 => match self.cartridge().mirroring() {
                // the mirroring is done by a brute force method even thoug there are more clever solutions.
                Mirroring::Horizontal => match address & 0x0FFF {
                    0x0000..0x0400 => { self.name_table[0][(address & 0x03FF) as usize] = data },
                    0x0400..0x0800 => { self.name_table[1][(address & 0x03FF) as usize] = data },
                    0x0800..0x0C00 => { self.name_table[0][(address & 0x03FF) as usize] = data },
                    0x0C00..0x1000 => { self.name_table[1][(address & 0x03FF) as usize] = data },
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
                Mirroring::Vertical => match address & 0x0FFF {
                    0x0000..0x0400 => { self.name_table[0][(address & 0x03FF) as usize] = data },
                    0x0400..0x0800 => { self.name_table[0][(address & 0x03FF) as usize] = data },
                    0x0800..0x0C00 => { self.name_table[1][(address & 0x03FF) as usize] = data },
                    0x0C00..0x1000 => { self.name_table[1][(address & 0x03FF) as usize] = data },
                    0x1000.. => unreachable!("after masking the address with 0x0FFF it can't be >= 0x1000."),
                },
            },

            // pallette memory
            0x3F00..0x4000 => {
                let masked_address = address & 0x001F;
                // todo: make sure this works
                let mirrored_address = masked_address % 32;
                // let mirrored_address = match masked_address {
                //     0x0010 => 0x0000,
                //     0x0014 => 0x0004,
                //     0x0018 => 0x0008,
                //     0x001C => 0x000C,
                //     _ => unreachable!(),
                // };

                self.palette[mirrored_address as usize] = data; 
            },

            0x4000.. => unreachable!("PPU bus only goes from has address range 0x0000..0x4000"),
        }
    }

    pub fn cpu_read_from_ppu(&mut self, address: u16) -> u8 {
        match address {
            CONTROL => { warn!("unimplemented. returning dummy 0 value."); 0 },
            MASK => { warn!("unimplemented. returning dummy 0 value."); 0 },
            STATUS => {
                let retrieved_data = (u8::from(self.ppu_controls.status) & 0xE0) | (self.ppu_controls.data_buffer & 0x1F);
                self.ppu_controls.status.set_vertical_blank(false);
                self.ppu_controls.writing_part = WritingAddressPart::default();
                retrieved_data
            },
            OAM_ADDRESS => { warn!("unimplemented. returning dummy 0 value."); 0 },
            OAM_DATA => { warn!("unimplemented. returning dummy 0 value."); 0 },
            SCROLL => { warn!("unimplemented. returning dummy 0 value."); 0 },
            PPU_ADDRESS => { error!("Read from address register even though it should't be allowed. Returning dummy 0."); 0 },
            PPU_DATA => {
                let mut retrieved_data = self.ppu_controls.data_buffer; // this will be returned.
                self.ppu_controls.data_buffer = self.ppu_read(self.ppu_controls.vram_address.into());

                if u16::from(self.ppu_controls.vram_address) >= 0x3F00 {
                    retrieved_data = self.ppu_controls.data_buffer;
                }

                // self.ppu_controls.vram_address += if self.ppu_controls.control.increment_mode() { 32 } else { 1 };
                self.ppu_controls.vram_address.increment(
                    if self.ppu_controls.control.increment_mode() { 32 } else { 1 }
                );

                retrieved_data
            },
            8.. => unreachable!("This function should only be called with mirroring from either Bus or Cartrige."),
        }
    }
    
    pub fn cpu_write_to_ppu(&mut self, address: u16, data: u8) {
        match address {
            CONTROL => {
                self.ppu_controls.control = data.into();
                self.ppu_controls.tram_address.set_nametable_x(self.ppu_controls.control.nametable_x());
                self.ppu_controls.tram_address.set_nametable_y(self.ppu_controls.control.nametable_y());
            },
            MASK => { self.ppu_controls.mask = data.into() },
            STATUS => panic!("You can't write to the status register."),
            OAM_ADDRESS => { error!("todo: finish this. no data was changed") },
            OAM_DATA => { error!("todo: finish this") },
            SCROLL => {
                use WritingAddressPart as WAP;
                match self.ppu_controls.writing_part {
                    WAP::High => {
                        self.ppu_controls.fine_x = data & 0x07;
                        self.ppu_controls.tram_address.set_coarse_x(data >> 3);
                    },
                    WAP::Low => {
                        self.ppu_controls.tram_address.set_fine_y(data & 0x07);
                        self.ppu_controls.tram_address.set_coarse_y(data >> 3);
                    },
                }

                self.ppu_controls.address_part_switch();
            },
            PPU_ADDRESS => {
                use WritingAddressPart as WAP;
                match self.ppu_controls.writing_part {
                    WAP::High => {
                        self.ppu_controls.tram_address = ((u16::from(self.ppu_controls.tram_address) & 0x00FF) | ((data as u16) << 8)).into();
                        // self.ppu_controls.tram_address = PPULoopy::from_bytes([data, self.ppu_controls.tram_address.into_bytes()[0]]);
                    },
                    WAP::Low => {
                        self.ppu_controls.tram_address = ((u16::from(self.ppu_controls.tram_address) & 0xFF00) | (data as u16)).into();
                        self.ppu_controls.vram_address = self.ppu_controls.tram_address;
                    },
                }
                
                self.ppu_controls.address_part_switch();
            },
            PPU_DATA => {
                self.ppu_write(self.ppu_controls.vram_address.into(), data);
                self.ppu_controls.vram_address.increment(
                    if self.ppu_controls.control.increment_mode() { 32 } else { 1 }
                );
            },
            8.. => unreachable!("This function should only be called with mirroring from either Bus or Cartrige."),
        }
    }
}


impl Bus {
    pub fn get_mut_cpu_ram(&mut self) -> &mut [u8; 2048] {
        &mut self.cpu_ram
    }

    pub fn get_mut_ppu_controls(&mut self) -> &mut PPUControlRegisters{
        &mut self.ppu_controls
    }
}