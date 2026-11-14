use super::*;

pub const CONTROL: u16 = 0;
pub const MASK: u16 = 1;
pub const STATUS: u16 = 2;
pub const OAM_ADDRESS: u16 = 3;
pub const OAM_DATA: u16 = 4;
pub const SCROLL: u16 = 5;
pub const PPU_ADDRESS: u16 = 6;
pub const PPU_DATA: u16 = 7;

impl CPU {
    pub fn cpu_read_from_ppu(&mut self, address: u16, read_only: bool) -> u8 {
        let ppu_controls = &mut self.ppu.ppu_controls;
        match address {
            CONTROL => { warn!("Reading from PPU_CONTROL register is not allowed. Returning dummy `0`."); 0 },

            MASK => { warn!("Reading from PPU_MASK register is not allowed. Returning dummy `0`."); 0 },

            STATUS => {
                let retrieved_data = (u8::from(ppu_controls.status) & 0xE0) | (ppu_controls.data_buffer & 0x1F);
                
                if !read_only {
                    ppu_controls.status.set_vertical_blank(false);
                    ppu_controls.writing_part = WritingAddressPart::default();
                }
                
                retrieved_data
            },

            OAM_ADDRESS => {
                warn!("Reading OAM address shouldn't be done. Returned actual value");
                self.ppu.oam_address
            },

            OAM_DATA => {
                self.ppu.oam[self.ppu.oam_address]
            },

            SCROLL => { warn!("unimplemented. returning dummy 0 value."); 0 },
            
            PPU_ADDRESS => { warn!("Reading from PPU_ADDRESS register is not allowed. Returning dummy `0`."); 0 },
            
            PPU_DATA => {
                let delayed_data = ppu_controls.data_buffer;
                let current_address: u16 = ppu_controls.vram_address.into();
                
                let new_data_buffer = self.ppu.read(current_address, &self.cartridge);
                let ppu_controls = &mut self.ppu.ppu_controls; // we needed to drop this value for a second so that Rust doesn't complain
                ppu_controls.data_buffer = new_data_buffer;
                
                ppu_controls.vram_address.increment(
                    if ppu_controls.control.increment_mode() { 32 } else { 1 }
                );

                if u16::from(ppu_controls.vram_address) >= 0x3F00 { // 0x3F00 = start of palette memory
                    new_data_buffer
                } else {
                    delayed_data
                }
            },

            8.. => unreachable!("This function should only be called with mirroring from either"),
        }
    }
    
    pub fn cpu_write_to_ppu(&mut self, address: u16, data: u8) {
        let ppu_controls = &mut self.ppu.ppu_controls;
        match address {
            CONTROL => {
                ppu_controls.control = data.into();
                ppu_controls.tram_address.set_nametable_x(ppu_controls.control.nametable_x());
                ppu_controls.tram_address.set_nametable_y(ppu_controls.control.nametable_y());
            },
            
            MASK => { ppu_controls.mask = data.into() },

            STATUS => warn!("Attempted to write {:02X} into status register.", data),

            OAM_ADDRESS => { self.ppu.oam_address = data },

            OAM_DATA => { self.ppu.oam[self.ppu.oam_address] = data },

            SCROLL => {
                use WritingAddressPart as WAP;
                match ppu_controls.writing_part {
                    WAP::High => {
                        ppu_controls.fine_x = data & 0x07;
                        ppu_controls.tram_address.set_coarse_x(data >> 3);
                    },
                    WAP::Low => {
                        ppu_controls.tram_address.set_fine_y(data & 0x07);
                        ppu_controls.tram_address.set_coarse_y(data >> 3);
                    },
                }

                ppu_controls.address_part_switch();
            },

            PPU_ADDRESS => {
                use WritingAddressPart as WAP;
                match ppu_controls.writing_part {
                    WAP::High => {
                        ppu_controls.tram_address = (
                            (u16::from(ppu_controls.tram_address) & 0x00FF) | (((data as u16) & 0x3F) << 8)
                        ).into();
                        // ppu_controls.tram_address = PPULoopy::from_bytes([data & 0x3F, ppu_controls.tram_address.into_bytes()[0]]);
                    },
                    WAP::Low => {
                        ppu_controls.tram_address = (
                            (u16::from(ppu_controls.tram_address) & 0xFF00) | (data as u16)
                        ).into();
                        ppu_controls.vram_address = ppu_controls.tram_address;
                    },
                }
                
                ppu_controls.address_part_switch();
            },

            PPU_DATA => {
                self.ppu.write(self.ppu.ppu_controls.vram_address.into(), data, &mut self.cartridge);
                self.ppu.ppu_controls.vram_address.increment(
                    if self.ppu.ppu_controls.control.increment_mode() { 32 } else { 1 }
                );
            },

            8.. => unreachable!("This function should only be called with mirroring."),
        }
    }
}