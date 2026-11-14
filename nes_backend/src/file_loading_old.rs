// crate imports
use crate::bit_operations::BitManipulation;

// std imports
use std::fs::File;
use std::path::Path;
use std::{fmt::{Debug, Display}, io::{self, Read}};

// third-party imports
use modular_bitfield::prelude::*;
use thiserror::Error;

// constansts
pub const PRG_BANK_SIZE: usize = 1 << 14;
pub const CHR_BANK_SIZE: usize = 1 << 13;

// source: https://www.nesdev.org/wiki/INES
#[derive(derive_getters::Getters)]
pub struct INesFile {
    header: INesFileHeader,
    trainer: Option<Trainer>,
    prg_rom_data: Vec<u8>, // todo: constraint this: [2<<13 == 16384; u8] * x
    chr_rom_data: Vec<u8>, // todo: constraint this: [2<<12 == 8192; u8] * y
    playchoice_inst_rom: Option<[u8; 8192]>, // todo: understand this
    playchoice_prom: Option<[u8; 32]>, // todo: understand this
}

// todo: make this more readable
#[derive(Debug, derive_getters::Getters)]
pub struct INesFileHeader {
    prg_rom_size: u8,
    chr_rom_size: u8,    

    // flags 6
    flags6: Flags6,
    // nametable_arrangement: NametableArrangement,
    // contains_battery_ram: bool,
    // trainer_present: bool,
    // alternative_nametable_layout: bool,
    
    // flags 7
    vs_unisystem: bool,
    playchoice_10: bool,
    nes20_format: bool,
    
    // mapper
    mapper_number: u8,

    // todo: finish the rest of the flags
    flags8: u8,
    flags9: u8,
    flags10: u8,
    
    // unused padding, but some dumpers populate it. Just for completenes.
    padding: [u8; 5]
}

type Trainer = [u8; 512];

impl INesFile {
    pub fn try_new_from_path(location: impl AsRef<Path>) -> Result<INesFile, FileDecodingError> {
        let mut file = File::open(location)?;

        // the header lives in the first 16 bytes of the file
        let mut header_bytes = [0u8; 16];
        file.read_exact(&mut header_bytes)?;
        let header = INesFileHeader::new_from_bytes(header_bytes)?;

        let trainer = if header.flags6.trainer_present() {
            let mut trainer_data = [0u8; 512];
            file.read_exact(&mut trainer_data)?;
            Some(trainer_data)
        } else {
            None
        }; 

        // PRG ROM (program ROM)
        // we *could* ommit initialization, but I'm not going to cause it's not a bottle neck.
        let mut prg_rom_data = vec![0u8; header.prg_rom_size as usize * PRG_BANK_SIZE];  
        file.read_exact(&mut prg_rom_data)?;

        // CHR ROM (character ROM)
        let mut chr_rom_data = vec![0u8; header.chr_rom_size as usize * CHR_BANK_SIZE];
        file.read_exact(&mut chr_rom_data)?;

        // Possible 8192 bytes for playchoice data (whatever it is)
        let playchoice = if header.playchoice_10 {
            let mut playchoice_data = [0u8; 8192];
            file.read_exact(&mut playchoice_data)?;
            Some(playchoice_data)
        } else {
            None
        };

        // this *should* be empty but I'm leaving it for debbugging purposes
        let mut rest = Vec::new();
        file.read_to_end(&mut rest)?;

        Ok(INesFile {
            header,
            trainer,
            prg_rom_data,
            chr_rom_data,
            playchoice_inst_rom: playchoice,
            playchoice_prom: None, // todo: what is this?
        })
    }
}


#[derive(Debug, Error)]
pub enum FileDecodingError {
    #[error("The first 4 bytes of the file don't match the standard 4-byte header. Expected `[N, E, S, 0x1A]`, but found `{0:?}` ")]
    ImproperHEaderError([u8; 4]),

    #[error("IO error")]
    IO(#[from] io::Error),
}



impl INesFileHeader {
    pub fn new_from_bytes(bytes: [u8; 16]) -> Result<INesFileHeader, FileDecodingError> {
        // it should follow this specific layout
        let nes_ascii = &bytes[0..4];
        
        // let flags6 = &bytes[6];
        let flags6 = Flags6::from(bytes[6]);

        // println!("raw sixth byte: {:08b}", bytes[6]);
        // println!("0th bit of that byte: {}", bytes[6].nth_bit::<u8>(0));
        // println!("NT arrangement read by a bitfield: {:?}", flags6_bitfield.nametable_arrangement());
        
        let flags7 = &bytes[7];
        let flags8 = &bytes[8];
        let flags9 = &bytes[9];
        let flags10 = &bytes[10];
        
        let padding: [u8; 5] = (&bytes[11..16]).try_into().expect("This slice is obviously 5 elements long.");

        // ---------- splitting and validation ----------
        const STANDARD_INES_HEADER: [u8; 4] = [b'N', b'E', b'S', 0x1A];
        if nes_ascii != STANDARD_INES_HEADER {
            return Err(FileDecodingError::ImproperHEaderError([nes_ascii[0], nes_ascii[1], nes_ascii[2], nes_ascii[3]]));
        }

        // todo: change these into proper structs using bitfields
        // flags 6
        // use NametableArrangement as NA;
        // let nametable_arrangement = if flags6.nth_bit::<u8>(0) == 0 { NA::Vertical } else { NA::Horizontal };
        // let contains_battery_ram = flags6.nth_flag(1);
        // let trainer_present = flags6.nth_flag(2);
        // let alternative_nametable_layout = flags6.nth_flag(3);
        // let mapper_lower_nibble = flags6 >> 4;

        // println!("NT arrangement read by a manual test: {:?}", nametable_arrangement);

        // flags 7
        let vs_unisystem = flags7.nth_flag(0);
        let playchoice_10 = flags7.nth_flag(1);
        let nes20_format = (flags7 & 0b0000_1100) >> 2 == 2;
        let mapper_upper_nibble = flags7 & 0xF0;

        // mapper
        let mapper_number = mapper_upper_nibble | flags6.mapper_lower_nibble();

        Ok(INesFileHeader {
            prg_rom_size: bytes[4],
            chr_rom_size: bytes[5],

            // flags 6
            // nametable_arrangement,
            // contains_battery_ram,
            // trainer_present,
            // alternative_nametable_layout,
            flags6,
            
            // flags 7
            vs_unisystem,
            playchoice_10,
            nes20_format,
            
            // mapper
            mapper_number,

            // of course these flags do have meanings
            // but they're beyond the scope of this simple emulator
            // todo: implement these flags
            flags8: flags8.clone(),
            flags9: flags9.clone(),
            flags10: flags10.clone(),
            
            // unused padding, but some dumpers populate it. Just for completenes I include it.
            padding,
        })
    }
}

#[modular_bitfield::bitfield]
#[repr(u8)]
#[derive(Debug)]
pub struct Flags6 {
    pub nametable_arrangement: NametableArrangement,
    pub contains_battery_ram: bool,
    pub trainer_present: bool,
    pub alternative_nametable_layout: bool,
    mapper_lower_nibble: B4,
}

#[derive(Debug, Specifier, Clone, Copy, PartialEq, Eq)]
#[bits = 1]
pub enum NametableArrangement {
    Vertical = 0,
    Horizontal = 1,
}

#[modular_bitfield::bitfield]
#[repr(u8)]
#[derive(Debug)]
struct Flags7 {
    vs_unisystem: bool,
    playchoice_10: bool,
    nes20_format: B2,
    mapper_upper_nibble: B4,
}