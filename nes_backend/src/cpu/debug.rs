use super::*;

// * debugging
impl CPU {
    pub fn get_ram(&self) -> &[u8; CPU_RAM_SIZE] {
        &self.ram
    }

    pub fn get_mut_ram(&mut self) -> &mut [u8; CPU_RAM_SIZE] {
        &mut self.ram
    }

    pub fn render_debug_pattern_table<P: PixelBuffer>(&mut self, screen: &mut P, pattern_table_side: PatternTable, pallete_id: u8) {
        self.ppu.render_debug_pattern_table(screen, pattern_table_side, pallete_id, &self.cartridge);
    }

    pub fn clock_until_next_instruction(&mut self) {
        self.clock();
        while self.cycles != 0 {
            self.clock();
        }
    }
}