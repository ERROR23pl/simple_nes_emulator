use super::*;
use crate::bit_operations::get_high_byte;

#[derive(Clone, Copy, PartialEq, Eq, Default, strum_macros::Display)]
pub enum AddressingMode {
    #[default]
    IMP,
    IMM,	
	ZP0,
    ZPX,	
	ZPY,
    REL,
	ABS,
    ABX,	
	ABY,
    IND,	
	IZX,
    IZY,
}

// todo: remove all & 0x00FF -- they are not needed in Rust
impl<P: PixelBuffer> CPU<P> {
    pub fn find_data(&mut self, mode: AddressingMode) -> bool {
        use AddressingMode as AM;
        match mode {
            AM::IMP => {
                // operation has to have access to accumulator because it might want to use it
                self.fetched = self.acc;

                false
            },

            AM::IMM => {
                self.addr_abs = self.program_counter;
                self.program_counter += 1;
                
                false
            },

            AM::ZP0 => {
                self.addr_abs = self.read(self.program_counter, false) as u16;	
                self.program_counter += 1;
                // self.addr_abs &= 0x00FF; // ? I'm fairly certain we don't need to do that here.
                
                false
            },

            AM::ZPX => {
                // self.addr_abs = self.read(self.program_counter, false) as u16 + self.reg_x as u16;
                self.addr_abs = self.read(self.program_counter, false).wrapping_add(self.reg_x) as u16;
                self.program_counter += 1;
                // self.addr_abs &= 0x00FF; // adding `reg_x` can make us cross the page boundry, which wraps around
                
                false
            },
            
            AM::ZPY => {
                // self.addr_abs = self.read(self.program_counter, false) as u16 + self.reg_y as u16;
                self.addr_abs = self.read(self.program_counter, false).wrapping_add(self.reg_y) as u16;
                self.program_counter += 1;
                // self.addr_abs &= 0x00FF; // adding `reg_y` can make us cross the page boundry, which wraps around

                false
            },

            // todo: perhaps I fucked up this relative addressing thing?
            AM::REL => {
                self.addr_rel = self.read(self.program_counter, false) as u16;
                self.program_counter += 1;

                if self.addr_rel & 0x80 != 0 {
                    self.addr_rel |= 0xFF00;
                }

                false
            },

            AM::ABS => {
                let low_byte = self.read(self.program_counter, false);
                let high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;

                self.addr_abs = glue_u8s(high_byte, low_byte);

                false
            },

            AM::ABX => {
                let low_byte = self.read(self.program_counter, false);
                let high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;

                self.addr_abs = glue_u8s(high_byte, low_byte) + self.reg_x as u16;

                // 0xFF00 grabs the high byte after adding x and compares it to the high_byte from input
                // let page_has_been_changed = self.addr_abs & 0xFF00 != (high_byte << 8);
                let page_has_been_changed = get_high_byte(self.addr_abs) != high_byte;
                
                page_has_been_changed
            },
            
            AM::ABY => {
                let low_byte = self.read(self.program_counter, false);
                let high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;
                
                self.addr_abs = glue_u8s(high_byte, low_byte) + self.reg_y as u16;
                
                // 0xFF00 grabs the high byte after adding x and compares it to the high_byte from input
                // let page_has_been_changed = self.addr_abs & 0xFF00 != (high_byte << 8);
                let page_has_been_changed = get_high_byte(self.addr_abs) != high_byte;

                page_has_been_changed
            },

            AM::IND => {
                let pointer_low_byte = self.read(self.program_counter, false);
                let pointer_high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;

                let pointer = glue_u8s(pointer_high_byte, pointer_low_byte);
                
                let address_low_byte = self.read(pointer, false);
                
                // ! The following is a bug in the NES hardware. If the low byte of the pointer
                // ! happens to be 0xFF then it points to the end of the page.
                // ! In this case to get the high byte of the address we have to cross the page boundry
                // ! and add 1 to the pointer. This should change the high byte of the pointer, but
                // ! because of the bug, it actually doesn't.
                // ! We have to emulate this behavior since developers knew about it
                // ! and every game expects this bug to be present
                let address_high_byte = if pointer_low_byte == 0x00FF {
                    self.read(pointer & 0xFF00, false) // mask pointer with 0xFF00 to simulate +1 not affecting the high_byte
                } else {
                    self.read(pointer + 1, false)
                };
                
                self.addr_abs = glue_u8s(address_high_byte, address_low_byte);

                false
            },

            // there's a pointer to the propper adress on the zero page in a location sepcified as an argument
            AM::IZX => {
                let address_at_page_zero = self.read(self.program_counter, false) as u16;
                self.program_counter += 1;

                let low_byte  = self.read((address_at_page_zero + self.reg_x as u16 + 0) & 0x00FF, false);
                let high_byte = self.read((address_at_page_zero + self.reg_x as u16 + 1) & 0x00FF, false);

                self.addr_abs = glue_u8s(high_byte, low_byte);
                
                false
            },

            // notice that this works a bit differently than IZX
            AM::IZY => {
                let address_at_page_zero = self.read(self.program_counter, false) as u16;
                self.program_counter += 1;

                let low_byte  = self.read((address_at_page_zero + 0) & 0x00FF, false);
                let high_byte = self.read((address_at_page_zero + 1) & 0x00FF, false);

                self.addr_abs = glue_u8s(high_byte, low_byte) + self.reg_y as u16;

                // if we cross the page boundry we need another clock cycle    
                get_high_byte(self.addr_abs) != high_byte
            },
        }
    }
}