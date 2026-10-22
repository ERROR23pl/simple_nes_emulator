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
                self.addr_abs &= 0x00FF; // ? I'm fairly certain we don't need to do that here.
                
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
                let low_byte = self.read(self.program_counter + 0, false);
                let high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;

                self.addr_abs = glue_u8s(high_byte, low_byte);

                false
            },

            AM::ABX => {
                let low_byte = self.read(self.program_counter + 0, false);
                let high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;

                self.addr_abs = glue_u8s(high_byte, low_byte).wrapping_add(self.reg_x as u16);

                let page_has_been_changed = get_high_byte(self.addr_abs) != high_byte;
                
                page_has_been_changed
            },
            
            AM::ABY => {
                let low_byte = self.read(self.program_counter, false);
                let high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;
                
                self.addr_abs = glue_u8s(high_byte, low_byte).wrapping_add(self.reg_y as u16);
                
                let page_has_been_changed = get_high_byte(self.addr_abs) != high_byte;

                page_has_been_changed
            },

            // this one is used only used with the JMP instruction
            AM::IND => {
                let pointer_low_byte = self.read(self.program_counter + 0, false);
                let pointer_high_byte = self.read(self.program_counter + 1, false);
                self.program_counter += 2;

                let pointer = glue_u8s(pointer_high_byte, pointer_low_byte);
                
                let address_low_byte = self.read(pointer, false);
                
                // ! The following is a bug in the NES hardware. If the low byte of the pointer
                // ! happens to be 0xFF then it points to the end of the page.
                // ! In this case to get the high byte of the address we have to add 1 to the pointer
                // ! and subsequently, cross the page boundry. This should change the high byte of the pointer,
                // ! but because of the bug, it actually doesn't.
                // ! We have to emulate this behavior since developers knew about it
                // ! and every game expects this bug to be present
                let address_high_byte = if pointer_low_byte == 0xFF {
                    // the +1 only affects the lower byte which becomes 0x00.
                    // the high byte stays the same. To get this result,
                    // mask pointer with 0xFF00
                    self.read(pointer & 0xFF00, false) 
                } else {
                    self.read(pointer + 1, false)
                };
                
                self.addr_abs = glue_u8s(address_high_byte, address_low_byte);

                false
            },

            // In this mode, the programmer supplies an 8-bit address
            // to a pointer. Since it's 8-bit, not 16-bit, it lives on page zero.
            // Then `reg_x` is actually used to offset that pointer.
            AM::IZX => {
                let pointer_address = self.read(self.program_counter, false) as u16;
                self.program_counter += 1;

                let low_byte  = self.read((pointer_address + self.reg_x as u16 + 0) & 0x00FF, false);
                let high_byte = self.read((pointer_address + self.reg_x as u16 + 1) & 0x00FF, false);

                self.addr_abs = glue_u8s(high_byte, low_byte);
                
                false
            },

            // In this mode, the programmer supplies an 8-bit address
            // to a pointer. Since it's 8-bit, not 16-bit, it lives on page zero.
            // * notice that this works a bit differently than IZX
            // This time `reg_y` is used as an offset to the memory the pointer points to.
            AM::IZY => {
                let pointer_address = self.read(self.program_counter, false) as u16;
                self.program_counter += 1;

                let low_byte  = self.read((pointer_address + 0) & 0x00FF, false);
                let high_byte = self.read((pointer_address + 1) & 0x00FF, false);

                self.addr_abs = glue_u8s(high_byte, low_byte) + self.reg_y as u16;

                let page_boundry_crossed = get_high_byte(self.addr_abs) != high_byte;

                page_boundry_crossed
            },
        }
    }
}