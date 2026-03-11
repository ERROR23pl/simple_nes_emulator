use std::{cell::RefCell, rc::Rc};

use crate::{
    bus::Bus, cartridge::Cartridge, cpu::CPU, ppu::PPU, rendering::PixelBuffer
};

pub struct NES<P: PixelBuffer> {
    cpu: CPU,
    ppu: PPU<P>,
    clock_count: usize,
}

impl<P: PixelBuffer> NES<P> {
    pub fn new(pixel_buffer: P, cartridge: Cartridge) -> Self {
        let bus = Rc::new(RefCell::new(Bus::new(cartridge)));

        NES {
            cpu: CPU::new(Rc::clone(&bus)),
            ppu: PPU::new(pixel_buffer, bus),
            clock_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.clock_count = 0;
    }

    // * PPU clock is 3x faster than CPU clock
    pub fn clock(&mut self) {
        self.ppu.clock();

        if self.clock_count % 3 == 0 {
            self.cpu.clock();
        }

        self.clock_count += 1;
    }
}