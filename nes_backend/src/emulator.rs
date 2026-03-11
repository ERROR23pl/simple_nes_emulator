use crate::rendering::PixelBuffer;
use crate::nes::NES;
use crate::cartridge::Cartridge;

// todo: delete this

pub trait EmulatorImplementation {
    fn run(&mut self);
    fn insert_cartrigde(&mut self, cartridge: Cartridge);
    fn reset(&mut self);
}

// represents the whole App state
pub struct Emulator<P: PixelBuffer> {
    nes: Option<NES<P>>,
    renderer: P,
}
