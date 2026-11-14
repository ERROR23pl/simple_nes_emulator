use crate::{
    bit_operations::glue_u8s, cartridge::Cartridge, cpu::{CPU, CPU_RAM_SIZE}, ppu::{OAM, PPU}, rendering::{DummyBuffer, PatternTable, PixelBuffer}
};


#[derive(derive_getters::Getters)]
pub struct NES {
    pub cpu: CPU,

    // DMA transfer
    dma_proper_clock_cycle_reached: bool,
    dma_data: u8,
    dma_address: u8,
    oam_debug_copy: OAM,


    // debug stuff
    clock_count: usize,
    frame_count: usize,
}

impl NES {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: CPU::new(cartridge, PPU::new()),

            clock_count: 0,
            frame_count: 0,

            dma_proper_clock_cycle_reached: false,
            dma_data: 0,
            dma_address: 0,
            oam_debug_copy: OAM::default(),
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.clock_count = 0;
    }

    pub fn clock<P: PixelBuffer>(&mut self, screen: &mut P) {
        self.cpu.ppu_clock(screen);
        
        // PPU clock is 3x faster than CPU clock
        if self.clock_count % 3 == 0 {
            // todo: HOLY FUCK that's ugly, please change this.
            if self.cpu.dma_transfer {
                if !self.dma_proper_clock_cycle_reached {
                    if self.clock_count % 2 != 0 {
                        self.dma_proper_clock_cycle_reached = true;
                    }
                } else {
                    if self.clock_count % 2 == 0 {
                        self.dma_data = self.cpu.read(glue_u8s(self.cpu.dma_page, self.dma_address), true);
                    } else {
                        self.cpu.ppu.oam[self.dma_address] = self.dma_data;
                       
                        self.dma_address = self.dma_address.wrapping_add(1);

                        if self.dma_address == 0x00 {
                            self.cpu.dma_transfer = false;
                            self.dma_proper_clock_cycle_reached = false;
                            self.oam_debug_copy = self.cpu.ppu.oam.clone();
                        }
                    }

                }
            } else {
                self.cpu.clock();
            }
        }

        if self.cpu.ppu.nmi {
            self.cpu.ppu.nmi = false;
            self.cpu.non_maskable_interupt_request();
        }

        self.clock_count += 1;
    }
}

// impl NES<DummyBuffer> {
//     pub fn new_no_display(cartridge: Cartridge) -> Self {
//         Self::new(DummyBuffer::default(), DummyBuffer::default(), cartridge)
//     }
// }

impl NES {
    pub fn get_mut_cpu_ram(&mut self) -> &mut [u8; CPU_RAM_SIZE] {
        self.cpu.get_mut_ram()
    }

    pub fn render_debug_pattern_table<P: PixelBuffer>(&mut self, screen: &mut P, pattern_table_side: PatternTable, pallete_id: u8) {
        self.cpu.render_debug_pattern_table(screen, pattern_table_side, pallete_id);
    }

    pub fn check_frame_complete_and_toggle(&mut self) -> bool {
        if self.cpu.ppu.frame_complete {
            self.cpu.ppu.frame_complete = false;
            self.frame_count += 1;
            true
        } else {
            false
        }
    }
}