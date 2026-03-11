use crate::emulator::EmulatorImplementation;

mod emulator;
mod file_loading;
mod bit_operations;
mod rendering;
mod implementations;
mod utils;

mod nes;
mod cpu;
mod bus;
mod ppu;
mod cartridge;
mod mapper;

const MARIO_PATH: &str = "./roms/Super Mario Bros. (World).nes";
const MARIO_3_PATH: &str = "./roms/Super Mario Bros. 3 (Europe).nes";
const ZELDA: &str = "./roms/Legend of Zelda, The (USA) (Rev 1) (Animal Crossing).nes";

const FILE_PATH: &str = MARIO_PATH;

fn main() -> std::io::Result<()> {
    let file = file_loading::INesFile::new_from_file_path(FILE_PATH)?;

    let mut emu = implementations::minifb::MiniFBEMulator::new();

    emu.run();

    // in emu.run() we wait for the cartridge insert
    // emu.run();
    
    Ok(())
}
