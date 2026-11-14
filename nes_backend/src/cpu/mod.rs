// submodules declaration
mod instructions;
mod addressing;
mod ppu_communication;
pub mod disassembler;
pub mod debug;

// crate imports
use crate::{
    bit_operations::*,
    cartridge::Cartridge,
    ppu::{PPU, WritingAddressPart},
    rendering::{PatternTable, PixelBuffer},
};
use addressing::*;
use instructions::*;

// 3rd party imports
use modular_bitfield::bitfield;
use log::{warn};

// constants
pub const CPU_RAM_SIZE: usize = 2 * KB;
pub const STACK_BASE: u16 = 0x0100;

#[derive(derive_getters::Getters)]
pub struct CPU {
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
    pub ppu: PPU,
    pub controlers: [u8; 2],
    controlers_state: [u8; 2],

    // DMA variables
    pub dma_page: u8,
    pub dma_transfer: bool,
    
    // helper variables
    fetched: u8,
    addr_abs: u16,
    addr_rel: u16,
    opcode: u8,
    cycles: u8,
}

impl CPU {
    pub fn new(cartridge: Cartridge, ppu: PPU) -> Self {
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
            controlers: [0, 0],
            controlers_state: [0, 0],

            // DMA
            dma_page: 0x00,
            dma_transfer: false,

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

// * instruction execution
impl CPU {
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
        let page_boundry_crossed = self.find_data(*instruction.mode());
        let operation_needs_cycle = self.execute(instruction);
        
        page_boundry_crossed && operation_needs_cycle
    }
}

// * hardware operations
impl CPU {
    pub fn reset(&mut self) {
        // Get address to set program counter to and set it
        self.addr_abs = 0xFFFC;
        self.program_counter = self.read_u16_at_addr_abs();

        
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
        let (pc_high, pc_low) = split_u16(self.program_counter);
        self.push_to_stack(pc_high);
        self.push_to_stack(pc_low);
        
        // push the status register to the stack
        self.set_flag(StatusFlag::Break, false);
        self.set_flag(StatusFlag::Unused, true);
        self.set_flag(StatusFlag::DisableInterupts, true);
        self.push_to_stack(self.status);

        // new program counter is located at a fixed address.
        // note: the adress is different depending on whether it was a
        // maskable or nonmaskable interrupt. I made this function generic
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
}

// * memory management
impl CPU {
    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..0x2000 => { self.ram[(address & 0x07FF) as usize] = data },
            0x2000..0x4000 => { self.cpu_write_to_ppu(address & 0x0007, data) },
            0x4014 => {
                self.dma_page = data;
                self.dma_transfer = true;
            }
            0x4016 | 0x4017 => {
                let which_controler = (address & 0x0001) as usize;
                self.controlers_state[which_controler] = self.controlers[which_controler]
            },
            0x4020..=0xFFFF => { self.cartridge.cpu_write(address, data); } // todo: is this correct?
            _ => { warn!("attempted to write to memory that wasn't yet implemented ({:04X}). Not doing anything.", address) },
        }
    }
    
    // reading ppu control registers can alter the state of the PPU
    // thus `read_only` flag is passed for debugging purposes
    // if one would want to read from them without modifying the state
    pub fn read(&mut self, address: u16, read_only: bool) -> u8 {
        // source: https://www.nesdev.org/wiki/CPU_memory_map
        match address {
            0x0000..0x2000 => self.ram[(address & 0x07FF) as usize],
            0x2000..0x4000 => self.cpu_read_from_ppu(address & 0x0007, read_only),
            0x4016 | 0x4017 => {
                let which_controler = (address & 0x0001) as usize;
                let most_significant_bit = (self.controlers_state[which_controler] & 0x80) >> 7;

                self.controlers_state[which_controler] <<= 1;

                most_significant_bit
            }
            
            // 0x
            0x4020..=0xFFFF => self.cartridge.cpu_read(address),

            _ => { warn!("Attempted to read from memory that wasn't yet implemented ({:04X}). Returning 0.", address); 0 },
        }
    }
}

// * helper functions for easier manipulation of CPU state
impl CPU {
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
   
    fn push_to_stack(&mut self, value: u8) {
        self.write(STACK_BASE + self.stack_pointer as u16, value);
        self.stack_pointer -= 1;
    }

    fn pop_from_stack(&mut self) -> u8 {
        self.stack_pointer += 1;
        self.read(STACK_BASE + self.stack_pointer as u16, false)
    }

    pub fn ppu_clock<P: PixelBuffer>(&mut self, screen: &mut P) {
        self.ppu.clock(screen, &mut self.cartridge);
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

// * even more helper functions
impl CPU {
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