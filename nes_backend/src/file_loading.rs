// std imports
use std::fs::File;
use std::path::Path;
use std::io::{self, Read};

// third-party imports
use modular_bitfield::prelude::*;
use thiserror::Error;

// constansts
pub const PRG_BANK_SIZE: usize = 1 << 14;
pub const CHR_BANK_SIZE: usize = 1 << 13;

// source: https://www.nesdev.org/wiki/INES
// An iNES file consists of the following sections, in order:
// - Header (16 bytes)
// - Trainer, if present (0 or 512 bytes)
// - PRG ROM data (16384 * x bytes)
// - CHR ROM data, if present (8192 * y bytes)
// - PlayChoice INST-ROM, if present (0 or 8192 bytes)
// - PlayChoice PROM, if present (16 bytes Data, 16 bytes CounterOut) (this is often missing; see PC10 ROM-Images for details)
//
// Some ROM-Images additionally contain a 128-byte (or sometimes 127-byte) title at the end of the file. 
#[derive(derive_getters::Getters)]
pub struct INesFile {
    header: INesFileHeader,
    trainer: Option<[u8; 512]>,
    prg_rom_data: Vec<u8>, // todo: constraint this: [2<<13 == 16384; u8] * x
    chr_rom_data: Vec<u8>, // todo: constraint this: [2<<12 == 8192; u8] * y
    playchoice_inst_rom: Option<[u8; 8192]>, // todo: understand this
    playchoice_prom: Option<[u8; 32]>, // todo: understand this
}

// The format of the header is as follows:
// Bytes 	Description
// -------------------------------------------------------------------------------------------------
// 0-3 	    Constant $4E $45 $53 $1A (ASCII "NES" followed by MS-DOS end-of-file)
// 4 	    Size of PRG ROM in 16 KB units
// 5 	    Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
// 6 	    Flags 6 – Mapper, mirroring, battery, trainer
// 7 	    Flags 7 – Mapper, VS/Playchoice, NES 2.0
// 8 	    Flags 8 – PRG-RAM size (rarely used extension)
// 9 	    Flags 9 – TV system (rarely used extension)
// 10 	    Flags 10 – TV system, PRG-RAM presence (unofficial, rarely used extension)
// 11-15 	Unused padding (should be filled with zero, but some rippers put their name across bytes 7-15) 

#[derive(Debug, derive_getters::Getters)]
pub struct INesFileHeader {
    prg_rom_size: u8,
    chr_rom_size: u8,    

    flags6: Flags6,
    flags7: Flags7,

    // todo: finish the rest of the flags
    flags8: u8,
    flags9: u8,
    flags10: u8,
    
    // unused padding, but some dumpers populate it. Just for completenes.
    padding: [u8; 5]
}




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
        let playchoice = if header.flags7.playchoice_10() {
            let mut playchoice_data = [0u8; 8192];
            file.read_exact(&mut playchoice_data)?;
            Some(playchoice_data)
        } else {
            None
        };

        // this *should* be empty but sometimes it's not
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
        // "Constant $4E $45 $53 $1A (ASCII "NES" followed by MS-DOS end-of-file)"
        const STANDARD_INES_HEADER_PATTERN: [u8; 4] = [b'N', b'E', b'S', 0x1A];
        
        if &bytes[0..4] != STANDARD_INES_HEADER_PATTERN {
            return Err(FileDecodingError::ImproperHEaderError([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }

        Ok(INesFileHeader {
            prg_rom_size: bytes[4],
            chr_rom_size: bytes[5],

            flags6: Flags6::from(bytes[6]),
            flags7: Flags7::from(bytes[7]),
            
            flags8: bytes[8],
            flags9: bytes[9],
            flags10: bytes[10],
            
            padding: (&bytes[11..16]).try_into()
                .expect("This slice is obviously 5 elements long."),
        })
    }

    pub fn mapper_number(&self) -> u8 {
        self.flags7.mapper_upper_nibble() << 4 |  self.flags6.mapper_lower_nibble() as u8
    }
}


#[modular_bitfield::bitfield]
#[repr(u8)]
#[derive(Debug)]
struct Flags6 {
    nametable_arrangement: NametableArrangement,
    contains_battery_ram: bool,
    trainer_present: bool,
    alternative_nametable_layout: bool,
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    // Sanity check to make sure I understand how modular bitfield works
    fn test_modular_bitfield() {
        let flags6 = Flags6::from(0b0000_0001);
        assert!(flags6.nametable_arrangement() == NametableArrangement::Horizontal);
        let flags6 = Flags6::from(0b0000_0010);
        assert!(flags6.contains_battery_ram() == true);
        
        let flags6 = Flags6::from(0b1010_0000);
        assert!(flags6.mapper_lower_nibble() as u8 == 0b0000_1010);
    }

    #[test]
    fn mapper_number_reading_test() {
        const SMB3_PATH: &'static str = "../nes_backend/roms/Super Mario Bros. 3 (Europe).nes";
        let smb3 = INesFile::try_new_from_path(SMB3_PATH);

        assert!(smb3.is_ok());
        
        if let Ok(mut file) = smb3 {
            assert!(file.header.mapper_number() == 4);
            file.header.flags7.set_mapper_upper_nibble(0b1010);
            assert!(file.header.mapper_number() == 0b1010_0100);
        } else {
            panic!("test failed, probably cause there was no file.");
        }        
    }
}