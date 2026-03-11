use super::*;
use addressing::AddressingMode;

pub const STACK_BASE: u16 = 0x0100;

#[derive(Clone, Copy, derive_getters::Getters)]
pub struct Instruction {
    type_: InstructionType,
    mode: AddressingMode,
    cycles: u8,
}


#[derive(Clone, Copy, Default, strum_macros::Display)]
pub enum InstructionType {
    ADC, AND, ASL, BCC, 
    BCS, BEQ, BIT, BMI, 
    BNE, BPL, BRK, BVC, 
    BVS, CLC, CLD, CLI, 
    CLV, CMP, CPX, CPY, 
    DEC, DEX, DEY, EOR, 
    INC, INX, INY, JMP, 
    JSR, LDA, LDX, LDY, 
    LSR, NOP, ORA, PHA, 
    PHP, PLA, PLP, ROL, 
    ROR, RTI, RTS, SBC, 
    SEC, SED, SEI, STA, 
    STX, STY, TAX, TAY, 
    TSX, TXA, TXS, TYA,

    #[default]    
    INVALID,
}

impl Default for Instruction {
    #[allow(unconditional_recursion)]
    fn default() -> Self {
        Instruction {
            ..Default::default()
        }
    }
}

pub const INSTRUCTION_LOOKUP: [Instruction; 256] = {
    use InstructionType as IT;
    use AddressingMode as AM; 
    [
        Instruction { type_: IT::BRK,     mode: AM::IMM, cycles: 7},
        Instruction { type_: IT::ORA,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 3},
        Instruction { type_: IT::ORA,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::ASL,     mode: AM::ZP0, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::PHP,     mode: AM::IMP, cycles: 3},
        Instruction { type_: IT::ORA,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::ASL,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::ORA,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::ASL,     mode: AM::ABS, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::BPL,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::ORA,     mode: AM::IZY, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::ORA,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::ASL,     mode: AM::ZPX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::CLC,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::ORA,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::ORA,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::ASL,     mode: AM::ABX, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::JSR,     mode: AM::ABS, cycles: 6},
        Instruction { type_: IT::AND,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::BIT,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::AND,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::ROL,     mode: AM::ZP0, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::PLP,     mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::AND,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::ROL,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::BIT,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::AND,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::ROL,     mode: AM::ABS, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::BMI,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::AND,     mode: AM::IZY, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::AND,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::ROL,     mode: AM::ZPX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::SEC,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::AND,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::AND,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::ROL,     mode: AM::ABX, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::RTI,     mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::EOR,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 3},
        Instruction { type_: IT::EOR,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::LSR,     mode: AM::ZP0, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::PHA,     mode: AM::IMP, cycles: 3},
        Instruction { type_: IT::EOR,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::LSR,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::JMP,     mode: AM::ABS, cycles: 3},
        Instruction { type_: IT::EOR,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::LSR,     mode: AM::ABS, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::BVC,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::EOR,     mode: AM::IZY, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::EOR,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::LSR,     mode: AM::ZPX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::CLI,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::EOR,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::EOR,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::LSR,     mode: AM::ABX, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::RTS,     mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::ADC,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 3},
        Instruction { type_: IT::ADC,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::ROR,     mode: AM::ZP0, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::PLA,     mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::ADC,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::ROR,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::JMP,     mode: AM::IND, cycles: 5},
        Instruction { type_: IT::ADC,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::ROR,     mode: AM::ABS, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::BVS,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::ADC,     mode: AM::IZY, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::ADC,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::ROR,     mode: AM::ZPX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::SEI,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::ADC,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::ADC,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::ROR,     mode: AM::ABX, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::STA,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::STY,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::STA,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::STX,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 3},
        Instruction { type_: IT::DEY,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::TXA,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::STY,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::STA,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::STX,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::BCC,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::STA,     mode: AM::IZY, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::STY,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::STA,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::STX,     mode: AM::ZPY, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::TYA,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::STA,     mode: AM::ABY, cycles: 5},
        Instruction { type_: IT::TXS,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::STA,     mode: AM::ABX, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::LDY,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::LDA,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::LDX,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::LDY,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::LDA,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::LDX,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 3},
        Instruction { type_: IT::TAY,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::LDA,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::TAX,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::LDY,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::LDA,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::LDX,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::BCS,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::LDA,     mode: AM::IZY, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::LDY,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::LDA,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::LDX,     mode: AM::ZPY, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::CLV,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::LDA,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::TSX,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::LDY,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::LDA,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::LDX,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::CPY,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::CMP,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::CPY,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::CMP,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::DEC,     mode: AM::ZP0, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::INY,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::CMP,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::DEX,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::CPY,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::CMP,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::DEC,     mode: AM::ABS, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::BNE,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::CMP,     mode: AM::IZY, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::CMP,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::DEC,     mode: AM::ZPX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::CLD,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::CMP,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::NOP,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::CMP,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::DEC,     mode: AM::ABX, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::CPX,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::SBC,     mode: AM::IZX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::CPX,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::SBC,     mode: AM::ZP0, cycles: 3},
        Instruction { type_: IT::INC,     mode: AM::ZP0, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 5},
        Instruction { type_: IT::INX,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::SBC,     mode: AM::IMM, cycles: 2},
        Instruction { type_: IT::NOP,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::CPX,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::SBC,     mode: AM::ABS, cycles: 4},
        Instruction { type_: IT::INC,     mode: AM::ABS, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::BEQ,     mode: AM::REL, cycles: 2},
        Instruction { type_: IT::SBC,     mode: AM::IZY, cycles: 5},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 8},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::SBC,     mode: AM::ZPX, cycles: 4},
        Instruction { type_: IT::INC,     mode: AM::ZPX, cycles: 6},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 6},
        Instruction { type_: IT::SED,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::SBC,     mode: AM::ABY, cycles: 4},
        Instruction { type_: IT::NOP,     mode: AM::IMP, cycles: 2},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 4},
        Instruction { type_: IT::SBC,     mode: AM::ABX, cycles: 4},
        Instruction { type_: IT::INC,     mode: AM::ABX, cycles: 7},
        Instruction { type_: IT::INVALID, mode: AM::IMP, cycles: 7},
    ]
};

// some of the following match statements
// are very repetetive. The following macros
// help with boilerplate and reuseability.
macro_rules! branch_if {
    ($flag:expr, $self:ident, $bool:expr) => {
        {
            if $self.get_flag($flag) == $bool {
                $self.cycles += 1;
                $self.addr_abs = $self.program_counter + $self.addr_rel;
                
                if ($self.addr_abs & 0xFF00) != ($self.program_counter & 0xFF00) {
                    $self.cycles += 1;
                }
                
                $self.program_counter = $self.addr_abs;
            }

            false
        }
    };
}   

macro_rules! compare {
    ($self:ident, $register:expr, $ret:literal) => {
        {
            $self.fetch();
            let temp = ($register as u16) - ($self.fetched as u16);
            $self.set_flag(StatusFlag::Carry, $register >= $self.fetched);
            $self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0x0000);
            $self.set_flag(StatusFlag::Negative, temp & 0x0080 != 1);

            $ret
        }
    };
}

macro_rules! incr_flags {
    ($self:ident, $register:expr) => {
        {
            $self.set_flag(StatusFlag::Zero, $register == 0x00);
            $self.set_flag(StatusFlag::Negative, $register & 0x80 != 0);
            
            false
        }
    };
}

macro_rules! load {
    ($self:ident, $register:expr) => {
        {
            $self.fetch();
            $register = $self.fetched;
            $self.set_flag(StatusFlag::Zero, $register == 0x00);
            $self.set_flag(StatusFlag::Negative, $register & 0x80 != 0);
            
            true
        }
    };
}

// todo: change all `& 0x80 != 0` to a function that checks if number is negative as i8
impl CPU {
    #[must_use]
    pub fn operate(&mut self, instr: Instruction) -> bool {
        use InstructionType as IT;
        
        match instr.type_ {
            IT::ADC => {
                self.fetch();
                
                let temp: u16 = self.acc as u16 + self.fetched as u16 + self.get_flag(StatusFlag::Carry) as u16;
                
                self.set_flag(StatusFlag::Carry, temp > 255);
                
                self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0);
                
                self.set_flag(
                    StatusFlag::Overflow,
                    (!((self.acc as u16) ^ (self.fetched) as u16) & ((self.acc as u16) ^ temp)) & 0x0080 != 0
                );
                
                self.set_flag(StatusFlag::Negative, temp & 0x80 != 0);
                
                self.acc = (temp & 0x00FF) as u8;
                
                true
            },

            IT::SBC => {
                self.fetch();
	
                let value = ((self.fetched as u16)) ^ 0x00FF;
                
                let temp = (self.acc as u16) + value + (self.get_flag(StatusFlag::Carry) as u16);
                self.set_flag(StatusFlag::Carry, temp & 0xFF00 != 0);
                self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0);
                self.set_flag(StatusFlag::Overflow, (temp ^ (self.acc as u16)) & (temp ^ value) & 0x0080 != 0);
                self.set_flag(StatusFlag::Negative, temp & 0x0080 != 0);
                self.acc = (temp & 0x00FF) as u8;

                true
            },

            IT::AND => {
                self.fetch();
                self.acc = self.acc & self.fetched;
                self.set_flag(StatusFlag::Carry, self.acc == 0);
                self.set_flag(StatusFlag::Negative, self.acc & 0x80 != 0);
                
                true
            },

            IT::ASL => {
                self.fetch();
                
                let temp = (self.fetched as u16) << 1;
                self.set_flag(StatusFlag::Carry, (temp & 0xFF00) > 0);
                self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0x00);
                self.set_flag(StatusFlag::Negative, temp & 0x80 != 0);
                
                if *INSTRUCTION_LOOKUP[self.opcode as usize].mode() == AddressingMode::IMP {
                    self.acc = (temp & 0x00FF) as u8;
                } else {
                    self.write(self.addr_abs, (temp & 0x00FF) as u8);
                }
                
                false
            },

            IT::BCC => { branch_if!(StatusFlag::Carry, self, false) },
            IT::BCS => { branch_if!(StatusFlag::Carry, self, true) },
            IT::BEQ => { branch_if!(StatusFlag::Zero, self, true) },
            
            IT::BIT => {
                self.fetch();
                let temp = (self.acc & self.fetched) as u16;
                self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0x0000);
                self.set_flag(StatusFlag::Negative, self.fetched & (1 << 7) != 0);
                self.set_flag(StatusFlag::Overflow, self.fetched & (1 << 6) != 0);
                
                false
            },
            
            IT::BMI => { branch_if!(StatusFlag::Negative, self, true) },
            IT::BNE => { branch_if!(StatusFlag::Zero, self, false) },
            IT::BPL => { branch_if!(StatusFlag::Negative, self, false) },

            IT::BRK => {
                self.program_counter += 1;
	
                self.set_flag(StatusFlag::DisableInterupts, true);
                self.write(STACK_BASE + (self.stack_pointer as u16), ((self.program_counter >> 8) & 0x00FF) as u8);
                self.stack_pointer -= 1;
                self.write(STACK_BASE + (self.stack_pointer as u16), ((self.program_counter) & 0x00FF) as u8);
                self.stack_pointer -= 1;
                
                self.set_flag(StatusFlag::Break, true);
                self.write(STACK_BASE + (self.stack_pointer as u16), self.status);
                self.stack_pointer -= 1;
                self.set_flag(StatusFlag::Break, false);

                self.program_counter = self.read(0xFFFE, false) as u16 | ((self.read(0xFFFF, false) as u16) << 8);
                
                false
            },

            IT::BVC => { branch_if!(StatusFlag::Overflow, self, false) },
            IT::BVS => { branch_if!(StatusFlag::Overflow, self, true) },

            IT::CLC => { self.set_flag(StatusFlag::Carry, false); false },
            IT::CLD => { self.set_flag(StatusFlag::DecimalMode, false); false },
            IT::CLI => { self.set_flag(StatusFlag::DisableInterupts, false); false },
            IT::CLV => { self.set_flag(StatusFlag::Overflow, false); false },

            IT::CMP => { compare!(self, self.acc, true) },
            IT::CPX => { compare!(self, self.reg_x, false) },
            IT::CPY => { compare!(self, self.reg_y, false) },
            
            IT::DEC => {
                self.fetch();
                let temp = (self.fetched - 1) as u16;
                
                self.write(self.addr_abs, (temp & 0x00FF) as u8);
                
                self.set_flag(StatusFlag::Carry, (temp & 0x00FF) == 0x0000);
                self.set_flag(StatusFlag::Negative, temp & 0x0080 != 0);
                
                false
            },
            
            IT::DEX => { self.reg_x -= 1; incr_flags!(self, self.reg_x) },
            IT::DEY => { self.reg_y -= 1; incr_flags!(self, self.reg_y) },
            
            IT::EOR => {
                self.fetch();
                self.acc = self.acc ^ self.fetched;	
                self.set_flag(StatusFlag::Zero, self.acc == 0x00);
                self.set_flag(StatusFlag::Negative, self.acc & 0x80 != 0);
                
                true
            },
            
            IT::INC => {
                self.fetch();
                let temp = (self.fetched + 1) as u16;
                
                self.write(self.addr_abs, (temp & 0x00FF) as u8);
                
                self.set_flag(StatusFlag::Carry, (temp & 0x00FF) == 0x0000);
                self.set_flag(StatusFlag::Negative, temp & 0x0080 != 0);
                
                false
            },
            
            IT::INX => { self.reg_x += 1; incr_flags!(self, self.reg_x) },
            IT::INY => { self.reg_y += 1; incr_flags!(self, self.reg_y) },

            IT::NOP => { matches!(self.opcode, 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC) },
            
            IT::JMP => { self.program_counter = self.addr_abs; false },
            IT::JSR => {
                self.program_counter -= 1;

                self.write(STACK_BASE + (self.stack_pointer as u16), ((self.program_counter >> 8) & 0x00FF) as u8);
                self.stack_pointer -= 1;
                self.write(STACK_BASE + (self.stack_pointer as u16), (self.program_counter & 0x00FF) as u8);
                self.stack_pointer -= 1;

                self.program_counter = self.addr_abs;

                false
            },

            IT::LDA => { load!(self, self.acc) },
            IT::LDX => { load!(self, self.reg_x) },
            IT::LDY => { load!(self, self.reg_y) },

            IT::LSR => {
                self.fetch();
                
                self.set_flag(StatusFlag::Carry, self.fetched & 0x0001 != 0);
                let temp = (self.fetched >> 1) as u16;	
                self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0x0000);
                self.set_flag(StatusFlag::Negative, temp & 0x0080 != 0);
                
                if *INSTRUCTION_LOOKUP[self.opcode as usize].mode() == AddressingMode::IMP {
                    self.acc = (temp & 0x00FF) as u8;
                } else {
                    self.write(self.addr_abs, (temp & 0x00FF) as u8);
                }
                
                false
            },

            IT::ORA => {
                self.fetch();

                self.acc = self.acc | self.fetched;
                self.set_flag(StatusFlag::Zero, self.acc == 0x00);
                self.set_flag(StatusFlag::Negative, self.acc & 0x80 != 0);

                true
            },

            IT::PHA => {
                self.write(STACK_BASE + (self.stack_pointer) as u16, self.acc);
                self.stack_pointer -= 1;
                
                false
            },

            IT::PHP => {
                self.write(STACK_BASE + (self.stack_pointer as u16), self.status | (StatusFlag::Carry as u8) | (StatusFlag::Unused as u8));
                
                self.set_flag(StatusFlag::Break, false);
                self.set_flag(StatusFlag::Unused, false);
                
                self.stack_pointer -= 1;
                
                false
            },

            IT::PLA => {
                self.stack_pointer += 1;
                self.acc = self.read(STACK_BASE + self.stack_pointer as u16, false);
                self.set_flag(StatusFlag::Zero, self.acc == 0x00);
                self.set_flag(StatusFlag::Negative, self.acc & 0x80 != 0);
                
                false
            },
            
            IT::PLP => {
                self.stack_pointer += 1;
                self.status = self.read(STACK_BASE + self.stack_pointer as u16, false);
                self.set_flag(StatusFlag::Unused, true);

                false
            },

            IT::ROL => {
                self.fetch();
                
                let temp = ((self.fetched << 1) as u16) | (self.get_flag(StatusFlag::Carry) as u16);
                
                self.set_flag(StatusFlag::Carry, temp & 0xFF00 != 0);
                self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0x0000);
                self.set_flag(StatusFlag::Negative, temp & 0x0080 != 0);
                
                if *INSTRUCTION_LOOKUP[self.opcode as usize].mode() == AddressingMode::IMP {
                    self.acc = (temp & 0x00FF) as u8;
                } else {
                    self.write(self.addr_abs, (temp & 0x00FF) as u8);
                }

                false
            },

            IT::ROR => {
                self.fetch();
                let temp = ((self.fetched << 1) as u16) | (self.get_flag(StatusFlag::Carry) as u16);
                
                self.set_flag(StatusFlag::Carry, temp & 0xFF00 != 0);
                self.set_flag(StatusFlag::Zero, (temp & 0x00FF) == 0x0000);
                self.set_flag(StatusFlag::Negative, temp & 0x0080 != 0);
                
                if *INSTRUCTION_LOOKUP[self.opcode as usize].mode() == AddressingMode::IMP {
                    self.acc = (temp & 0x00FF) as u8;
                } else {
                    self.write(self.addr_abs, (temp & 0x00FF) as u8);
                }
                
                false
            },

            IT::RTI => {
                self.stack_pointer += 1;

                self.status = self.read(STACK_BASE + self.stack_pointer as u16, false);
                self.status &= !(StatusFlag::Break as u8);
                self.status &= !(StatusFlag::Unused as u8);

                self.stack_pointer += 1;
                
                self.program_counter = self.read(STACK_BASE + self.stack_pointer as u16, false) as u16;
                self.stack_pointer += 1;
                
                self.program_counter |= (self.read(STACK_BASE + self.stack_pointer as u16, false) as u16) << 8;
                
                false
            },
            
            IT::RTS => {
                self.stack_pointer += 1;
                self.program_counter = self.read(STACK_BASE + self.stack_pointer as u16, false) as u16;
                self.stack_pointer += 1;
                self.program_counter |= (self.read(STACK_BASE + self.stack_pointer as u16, false) as u16) << 8;
                
                self.program_counter += 1;
                
                false
            },

            IT::SEC => { self.set_flag(StatusFlag::Carry, true); false },
            IT::SED => { self.set_flag(StatusFlag::DecimalMode, true); false },
            IT::SEI => { self.set_flag(StatusFlag::DisableInterupts, true); false },
            
            IT::STA => { self.write(self.addr_abs, self.acc); false },
            IT::STX => { self.write(self.addr_abs, self.reg_x); false },
            IT::STY => { self.write(self.addr_abs, self.reg_y); false },
            
            IT::TAX => {
                self.reg_x = self.acc;
                self.set_flag(StatusFlag::Zero, self.reg_x == 0x00);
                self.set_flag(StatusFlag::Negative, self.reg_x & 0x80 != 0);
                false
            },

            IT::TAY => {
                self.reg_y = self.acc;
                self.set_flag(StatusFlag::Zero, self.reg_y == 0x00);
                self.set_flag(StatusFlag::Negative, self.reg_y & 0x80 != 0);
                false
            },

            IT::TSX => {
                self.reg_x = self.stack_pointer;
                self.set_flag(StatusFlag::Zero, self.reg_x == 0x00);
                self.set_flag(StatusFlag::Negative, self.reg_x & 0x80 != 0);
                false
            },

            IT::TXA => {
                self.acc = self.reg_x;
                self.set_flag(StatusFlag::Zero, self.acc == 0x00);
                self.set_flag(StatusFlag::Negative, self.acc & 0x80 != 0);
                false
            },

            IT::TXS => {
                self.stack_pointer = self.reg_x;
                false
            },

            IT::TYA => {
                self.acc = self.reg_y;
                self.set_flag(StatusFlag::Zero, self.acc == 0x00);
                self.set_flag(StatusFlag::Negative, self.acc & 0x80 != 0);

                false
            },

            IT::INVALID => false,
        }
    }    
}
