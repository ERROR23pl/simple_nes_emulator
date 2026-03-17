mod instructions;
mod addressing;
pub mod disassembler;

// use std::{u16};

use crate::{
    bit_operations::*, bus::Bus, ppu::PPU, utils::Shared
};
use addressing::*;
use instructions::*;


#[derive(derive_getters::Getters)]
pub struct CPU {
    // actual registers on the 6502
    acc: u8,
    reg_x: u8,
    reg_y: u8,
    stack_pointer: u8,
    program_counter: u16,
    status: u8,

    // helper variables
    fetched: u8,
    addr_abs: u16,
    addr_rel: u16,
    opcode: u8,
    cycles: u8,

    // connection to the bus
    bus: Shared<Bus>,
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

impl CPU {
    pub fn new(bus: Shared<Bus>) -> Self {
        let mut cpu = CPU {
            // actual registers on the 6502
            acc: 0,
            reg_x: 0,
            reg_y: 0,
            stack_pointer: 0,
            program_counter: 0,
            status: 0,

            // helper variables
            fetched: 0,
            addr_abs: 0,
            addr_rel: 0,
            opcode: 0,
            cycles: 0,

            bus: bus,
        };

        // todo: uncomment when this stops crushing
        cpu.reset();

        cpu
    }
 
}

// * Core functionality
impl CPU {
    fn set_flag(&mut self, flag: StatusFlag, value: bool) {
        if value {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    fn get_flag(&self, flag: StatusFlag) -> bool {
        self.status & (flag as u8) != 0
    }

    fn read(&self, address: u16, read_only: bool) -> u8 {
        self.bus.borrow().cpu_read(address, read_only)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus.borrow_mut().cpu_write(address, data);        
    }

    // returns `true` if the execution needs another cycle
    fn execute(&mut self, instruction: Instruction) -> bool {
        let data_needs_cycle = self.find_data(*instruction.mode());
        let operation_needs_cycle = self.operate(instruction);

        data_needs_cycle && operation_needs_cycle
    }

    pub fn clock(&mut self) {
        if self.cycles == 0 {
            self.opcode = self.read(self.program_counter, false);
            self.program_counter += 1;

            let instruction = INSTRUCTION_LOOKUP[self.opcode as usize];
            
            self.cycles = instruction.cycles();
            self.cycles += self.execute(instruction) as u8; // add one cycle if returns true
        }

        self.cycles -= 1;
    }

    // pub fn ppu_clock(&mut self) {
    //     self.bus.ppu_clock()
    // }

    pub fn reset(&mut self) {
        // Get address to set program counter to
        self.addr_abs = 0xFFFC;
        let low_byte = self.read(self.addr_abs, false);
        let high_byte = self.read(self.addr_abs + 1, false);

        // Set it
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
        // Push the program counter to the stack. It's 16-bits dont
        // forget so that takes two pushes
        self.write(STACK_BASE + (self.stack_pointer as u16), ((self.program_counter >> 8) & 0x00FF) as u8);
        self.stack_pointer -= 1;
        self.write(STACK_BASE + (self.program_counter as u16), (self.program_counter & 0x00FF) as u8);
        self.stack_pointer -= 1;
        
        // Then Push the status register to the stack
        self.set_flag(StatusFlag::Break, false);
        self.set_flag(StatusFlag::Unused, true);
        self.set_flag(StatusFlag::DisableInterupts, true);
        
        self.stack_pointer -= 1;
        self.write(STACK_BASE + (self.stack_pointer as u16), self.status);

        // Read new program counter location from fixed address
        self.addr_abs = ADR;
        // let low_byte = self.read(self.addr_abs, false);
        // let high_byte = self.read(self.addr_abs + 1, false);
        // self.program_counter = glue_u8s(high_byte, low_byte);
        self.program_counter = self.read_addr_abs_twice()
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

    pub fn fetch(&mut self) -> u8 {
        let instruction = INSTRUCTION_LOOKUP[self.opcode as usize];
        
        if *instruction.mode() != AddressingMode::IMP {
            self.fetched = self.read(self.addr_abs, false);
        }
        
        self.fetched
    }
}

// * helper functions
impl CPU {
    fn set_flags(&mut self, flags: &[StatusFlag], value: bool) {
        let combined_flags = flags.iter().fold(0u8, |acc, x| acc | *x as u8);

        if value {
            self.status |= combined_flags;
        } else {
            self.status &= !combined_flags;
        }
    }

    fn read_next_and_clock(&mut self) -> u8 {
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
    fn read_addr_abs(&self) -> u8 {
        self.read(self.addr_abs, false)
    }

    #[inline]
    fn read_addr_abs_twice(&self) -> u16 {
        let low_byte = self.read(self.addr_abs, false);
        let high_byte = self.read(self.addr_abs + 1, false);

        glue_u8s(high_byte, low_byte)
    }
}
