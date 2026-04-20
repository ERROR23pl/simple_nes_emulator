use std::fmt::Display;

use super::*;
use crate::{bit_operations, cartridge::Cartridge, file_loading::INesFile, utils};

use log::debug;

pub enum Argument {
    Small(u8),
    Big(u16),
}

#[derive(derive_getters::Getters)]
pub struct DisassembledInstruction {
    address: usize,
    name: InstructionType,
    mode: AddressingMode,
    argument: Option<Argument>,
}

impl Display for DisassembledInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:04X} | {}", self.address, self.name)?;
        match self.argument {
            None => {
                write!(f, "      ")?;
            },
            Some(Argument::Small(arg)) => {
                match self.mode() {
                    AddressingMode::REL => { write!(f, " ${:04}", arg as i8)?; },
                    _ => { write!(f, "   ${:02X}", arg)?; },
                }
            },
            Some(Argument::Big(arg)) => {
                write!(f, " ${:04X}", arg)?;
            },
        };

        write!(f, "  {{{}}}", self.mode)?;
        Ok(())
    }
}

impl Cartridge {
    pub fn disassemble(&self) -> Vec<DisassembledInstruction> {
        let bytes = self.prg_memory();
        let mut result = Vec::new();

        let mut address = 0;
        while address < bytes.len() - 2 {
            let instr = INSTRUCTION_LOOKUP[bytes[address] as usize];

            use AddressingMode as AM;
            result.push(DisassembledInstruction {
                address,
                name: *instr.type_(),
                mode: *instr.mode(),
                argument: match instr.mode() {
                    AM::IMP => None,
                    AM::IMM | AM::ZP0 | AM::ZPX | AM::ZPY | AM::REL | AM::IZX | AM::IZY => {
                        address += 1;
                        let value = bytes[address];
                        Some(Argument::Small(value))
                    },
                    AM::ABS | AM::ABX | AM::ABY | AM::IND => {
                        address += 1;
                        let low_byte = bytes[address]; 
                        address += 1;
                        let high_byte = bytes[address];
                        Some(Argument::Big(bit_operations::glue_u8s(high_byte, low_byte)))
                    },
                },
            });

            address += 1;
        }

        result
    }
}