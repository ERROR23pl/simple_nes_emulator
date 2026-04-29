// submodules declaration
mod instructions;
mod addressing;
pub mod disassembler;

// crate imports
use crate::{
    bit_operations::*,
    cartridge::Cartridge,
    ppu::{PPU, ppu_controls::WritingAddressPart},
    rendering::{PatternTable, PixelBuffer},
};
use addressing::*;
use instructions::*;

// 3rd party imports
use modular_bitfield::bitfield;
use log::{error, warn};

// constants
pub const CPU_RAM_SIZE: usize = 2 * KB;

pub const CONTROL: u16 = 0;
pub const MASK: u16 = 1;
pub const STATUS: u16 = 2;
pub const OAM_ADDRESS: u16 = 3;
pub const OAM_DATA: u16 = 4;
pub const SCROLL: u16 = 5;
pub const PPU_ADDRESS: u16 = 6;
pub const PPU_DATA: u16 = 7;


#[derive(derive_getters::Getters)]
pub struct CPU<P: PixelBuffer> {
    // actual registers on the 6502
    acc: u8,
    reg_x: u8,
    reg_y: u8,
    stack_pointer: u8,
    program_counter: u16,
    status: u8,

    // CPU bus (on board elements controled or accessed by CPU)
    ram: [u8; CPU_RAM_SIZE],
    cartridge: Cartridge,
    pub ppu: PPU<P>,
    
    // helper variables
    fetched: u8,
    addr_abs: u16,
    // addr_rel: i8, // used to be `addr_rel: u16`
    addr_rel: u16,
    opcode: u8,
    cycles: u8,
}

impl<P: PixelBuffer> CPU<P> {
    pub fn new(cartridge: Cartridge, ppu: PPU<P>) -> Self {
        let mut cpu = CPU {
            // actual registers on the 6502
            acc: 0,
            reg_x: 0,
            reg_y: 0,
            stack_pointer: 0,
            program_counter: 0,
            status: 0,
            
            // cpu bus
            ram: [0; CPU_RAM_SIZE],
            cartridge,
            ppu,

            // helper variables
            fetched: 0,
            addr_abs: 0,
            addr_rel: 0,
            opcode: 0,
            cycles: 0,
        };

        cpu.reset();

        cpu
    }
 
}

// for now it's used only for printing
#[bitfield(bits=8)]
#[derive(Debug)]
pub struct Status {
    carry: bool,
    zero: bool,
    disableinterupts: bool,
    decimalmode: bool,
    break_: bool,
    unused: bool,
    overflow: bool,
    negative: bool,
}

macro_rules! RAM_RANGE { () => { 0x0000..0x2000 }; }
macro_rules! PPU_RANGE { () => { 0x2000..0x4000 }; }
macro_rules! PROGRAM_ROM_RANGE { () => { 0x4020..=0xFFFF }; }


impl<P: PixelBuffer> CPU<P> {
    fn set_flag(&mut self, flag: StatusFlag, value: bool) {
        if value {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    pub fn get_flag(&self, flag: StatusFlag) -> bool {
        self.status & (flag as u8) != 0
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            RAM_RANGE!() => { self.ram[(address & 0x07FF) as usize] = data },
            PPU_RANGE!() => { self.cpu_write_to_ppu(address & 0x0007, data) },
            PROGRAM_ROM_RANGE!() => { self.cartridge.cpu_write(address, data); } // todo: is this correct?
            _ => { warn!("attempted to write to memory that wasn't yet implemented ({:04X}). Not doing anything.", address) },
        }
    }
    
    // reading ppu control registers can alter the state of the PPU
    // thus `read_only` flag is passed for debugging purposes
    // if one would want to read from them without modifying the state
    pub fn read(&mut self, address: u16, read_only: bool) -> u8 {
        match address {
            RAM_RANGE!() => self.ram[(address & 0x07FF) as usize],
            PPU_RANGE!() => self.cpu_read_from_ppu(address & 0x0007, read_only),
            PROGRAM_ROM_RANGE!() => self.cartridge.cpu_read(address), // todo: is this correct?
            _ => { warn!("Attempted to read from memory that wasn't yet implemented ({:04X}). Returning 0.", address); 0 },
        }
    }
    
    fn cpu_read_from_ppu(&mut self, address: u16, read_only: bool) -> u8 {
        let ppu_controls = &mut self.ppu.ppu_controls;
        match address {
            CONTROL => { error!("Reading from PPU_CONTROL register is not allowed. Returning dummy `0`."); 0 },

            MASK => { error!("Reading from PPU_MASK register is not allowed. Returning dummy `0`."); 0 },

            STATUS => {
                let retrieved_data = (u8::from(ppu_controls.status) & 0xE0) | (ppu_controls.data_buffer & 0x1F);
                
                if !read_only {
                    ppu_controls.status.set_vertical_blank(false);
                    ppu_controls.writing_part = WritingAddressPart::default();
                }
                
                retrieved_data
            },

            OAM_ADDRESS => { warn!("unimplemented. returning dummy 0 value."); 0 },

            OAM_DATA => { warn!("unimplemented. returning dummy 0 value."); 0 },

            SCROLL => { warn!("unimplemented. returning dummy 0 value."); 0 },
            
            PPU_ADDRESS => { error!("Reading from PPU_ADDRESS register is not allowed. Returning dummy `0`."); 0 },
            
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

            STATUS => error!("Attempted to write {:02X} into status register.", data),

            OAM_ADDRESS => { warn!("OAM_ADDRESS write unimplemented") },

            OAM_DATA => { warn!("OAM_DATA write unimplemented") },

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
    
    pub fn clock(&mut self) {
        if self.cycles == 0 {
            self.opcode = self.read(self.program_counter, false);
            self.program_counter += 1;

            let instruction = INSTRUCTION_LOOKUP[self.opcode as usize];
            
            self.cycles = instruction.cycles();
            self.cycles += self.find_data_and_execute(instruction) as u8; // add one cycle if returns true
        }

        self.cycles -= 1;
    }
    
    // returns `true` if the execution needs another cycle
    fn find_data_and_execute(&mut self, instruction: Instruction) -> bool {
        let data_needs_cycle = self.find_data(*instruction.mode());
        let operation_needs_cycle = self.execute(instruction);
        
        data_needs_cycle && operation_needs_cycle
    }

    pub fn reset(&mut self) {
        // Get address to set program counter to and set it
        self.addr_abs = 0xFFFC;
        let low_byte = self.read(self.addr_abs, false); // todo: I have a function for this self.read_addr_abs_twice()
        let high_byte = self.read(self.addr_abs + 1, false);
        self.program_counter = glue_u8s(high_byte, low_byte);
        
        // Reset internal registers
        self.acc = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.stack_pointer = 0xFD;
        self.status = 0u8 | StatusFlag::Unused as u8;

        // Clear internal helper variables
        self.addr_rel = 0;
        self.addr_abs = 0;
        self.fetched = 0;

        // Reset takes time
        self.cycles = 8;
    }

    #[inline]
    fn interrupt<const ADR: u16>(&mut self) {
        // push the program counter to the stack.
        let (program_counter_high, program_counter_low) = split_u16(self.program_counter);
        self.write(STACK_BASE + (self.stack_pointer as u16), program_counter_high);
        self.stack_pointer -= 1;
        self.write(STACK_BASE + (self.stack_pointer as u16), program_counter_low);
        self.stack_pointer -= 1;
        
        // push the status register to the stack
        self.set_flag(StatusFlag::Break, false);
        self.set_flag(StatusFlag::Unused, true);
        self.set_flag(StatusFlag::DisableInterupts, true);
        
        self.write(STACK_BASE + (self.stack_pointer as u16), self.status);
        self.stack_pointer -= 1;

        // new program counter is located at a fixed address.
        // note: the adress is different depending on whether it was a
        // maskable or not interrupt. I made this function generic
        // over that adress to not repeat code
        self.addr_abs = ADR;
        self.program_counter = self.read_u16_at_addr_abs()
    }

    pub fn interupt_request(&mut self) {
        if !self.get_flag(StatusFlag::DisableInterupts) {
            self.interrupt::<0xFFFE>();
            self.cycles = 7;
        } 
    }

    pub fn non_maskable_interupt_request(&mut self) {
        self.interrupt::<0xFFFA>();
        self.cycles = 8;
    }

    

    pub fn ppu_clock(&mut self) {
        self.ppu.clock(&mut self.cartridge);
    }
}

#[derive(Clone, Copy)]
pub enum StatusFlag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    DisableInterupts = 1 << 2,
    DecimalMode = 1 << 3,
    Break = 1 << 4,
    Unused = 1 << 5,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

// * helper functions
impl<P: PixelBuffer> CPU<P> {
    fn set_flags(&mut self, flags: &[StatusFlag], value: bool) {
        let combined_flags = flags.iter().fold(0u8, |acc, x| acc | *x as u8);

        if value {
            self.status |= combined_flags;
        } else {
            self.status &= !combined_flags;
        }
    }

    fn read_next_u8_and_clock(&mut self) -> u8 {
        self.program_counter += 1;
        self.read(self.program_counter - 1, false)
    }

    fn read_next_u16_and_clock(&mut self) -> u16 {
        let low_byte = self.read(self.program_counter, false);
        let high_byte = self.read(self.program_counter + 1, false);

        self.program_counter += 2;

        glue_u8s(high_byte, low_byte)
    }

    #[inline]
    fn read_u8_at_addr_abs(&mut self) -> u8 {
        self.read(self.addr_abs, false)
    }

    #[inline]
    fn read_u16_at_addr_abs(&mut self) -> u16 {
        let low_byte = self.read(self.addr_abs, false);
        let high_byte = self.read(self.addr_abs + 1, false);

        glue_u8s(high_byte, low_byte)
    }
}

// * debugging
impl<P: PixelBuffer> CPU<P> {
    pub fn get_ram(&self) -> &[u8; CPU_RAM_SIZE] {
        &self.ram
    }

    pub fn get_mut_ram(&mut self) -> &mut [u8; CPU_RAM_SIZE] {
        &mut self.ram
    }

    pub fn render_debug_pattern_table(&mut self, pattern_table_side: PatternTable, pallete_id: u8) {
        self.ppu.render_debug_pattern_table(pattern_table_side, pallete_id, &self.cartridge);
    }

    pub fn clock_until_next_instruction(&mut self) {
        self.clock();
        while self.cycles != 0 {
            self.clock();
        }
    }
}