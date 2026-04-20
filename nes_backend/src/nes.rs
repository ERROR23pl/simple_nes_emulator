use crate::{
    cartridge::Cartridge, cpu::{CPU, CPU_RAM_SIZE}, ppu::PPU, rendering::{PatternTable, PixelBuffer}
};


#[derive(derive_getters::Getters)]
pub struct NES<P: PixelBuffer> {
    pub cpu: CPU<P>,
    clock_count: usize,
}

impl<P: PixelBuffer> NES<P> {
    pub fn new(pixel_buffer: P, debug_pattern_screen: P, cartridge: Cartridge) -> Self {
        Self {
            cpu: CPU::new(cartridge, PPU::new(pixel_buffer, debug_pattern_screen)),
            clock_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.clock_count = 0;
    }

    // * PPU clock is 3x faster than CPU clock
    pub fn clock(&mut self) {
        self.cpu.ppu_clock();

        if self.clock_count % 3 == 0 {
            self.cpu.clock();
        }

        if self.cpu.ppu.nmi {
            self.cpu.ppu.nmi = false;
            self.cpu.non_maskable_interupt_request();
        }

        self.clock_count += 1;
    }
}

impl<P: PixelBuffer> NES<P> {
    pub fn get_mut_cpu_ram(&mut self) -> &mut [u8; CPU_RAM_SIZE] {
        self.cpu.get_mut_ram()
    }

    pub fn render_debug_pattern_table(&mut self, pattern_table_side: PatternTable, pallete_id: u8) {
        self.cpu.render_debug_pattern_table(pattern_table_side, pallete_id);
    }
}