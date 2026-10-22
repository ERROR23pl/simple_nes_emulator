// module declarations
pub mod display;

use crate::app::RunningMode;

// crate imports
use super::App;
use display::BasicPixelBuffer;
use nes_backend::{cartridge::{self, Cartridge, UnimplementedMapperError}, file_loading::{self, FileDecodingError, INesFile}, nes::NES, rendering::{PatternTable, PixelBuffer}};
use thiserror::Error;

// std imports
use std::{io, path::PathBuf};





impl App {
    pub fn update_emulator(&mut self) {
        let Some(screen) = self.nes_screen.as_mut() else {
            return;
        };

        let nes = self.nes.as_mut().expect("I've already checked if it's Some(_) on the call site.");

        match self.debug_state.running_mode {
            RunningMode::CyclesPerFrame(speed) => {
                for _ in 0..speed {
                    nes.clock();

                    if nes.check_frame_complete_and_toggle() {
                        screen.update_with_pixel_buffer(&nes.cpu().ppu().screen);
                        if let (true, Some(pattern_screen)) = (self.app_settings.show.pattern_memory, self.pattern_screen.as_mut()) {
                            nes.render_debug_pattern_table(PatternTable::Left, self.debug_state.chosen_debug_palette);
                            nes.render_debug_pattern_table(PatternTable::Right, self.debug_state.chosen_debug_palette);
                            pattern_screen.update_with_pixel_buffer(&nes.cpu.ppu.pattern_table_screen);
                        }
                    };
                };
            },
            RunningMode::StepByStep => {
                if nes.check_frame_complete_and_toggle() {
                    screen.update_with_pixel_buffer(&nes.cpu().ppu().screen);
                    
                    if let (true, Some(pattern_screen)) = (self.app_settings.show.pattern_memory, self.pattern_screen.as_mut()) {
                        nes.render_debug_pattern_table(PatternTable::Left, self.debug_state.chosen_debug_palette);
                        nes.render_debug_pattern_table(PatternTable::Right, self.debug_state.chosen_debug_palette);
                        pattern_screen.update_with_pixel_buffer(&nes.cpu.ppu.pattern_table_screen);
                    }
                };
            },
            RunningMode::Normal => {
                // clock nes until the ppu finishes drawing the frame
                while !nes.check_frame_complete_and_toggle() { nes.clock() };
                screen.update_with_pixel_buffer(&nes.cpu().ppu().screen);

                // if we want to render pattern memory
                if let (true, Some(pattern_screen)) = (self.app_settings.show.pattern_memory, self.pattern_screen.as_mut()) {
                    nes.render_debug_pattern_table(PatternTable::Left, self.debug_state.chosen_debug_palette);
                    nes.render_debug_pattern_table(PatternTable::Right, self.debug_state.chosen_debug_palette);
                    pattern_screen.update_with_pixel_buffer(&nes.cpu.ppu.pattern_table_screen);
                }
            }
        }
    }

    
}

