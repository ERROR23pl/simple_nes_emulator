// module declarations
pub mod display;

use crate::app::RunningMode;

// crate imports
use super::App;
use display::BasicPixelBuffer;
use nes_backend::{cartridge::{self, Cartridge, UnimplementedMapperError}, file_loading::{self, FileDecodingError, INesFile}, nes::NES, rendering::{PatternTable, PixelBuffer}};
use thiserror::Error;

// std imports
use std::{collections::VecDeque, io, path::PathBuf};


// third-party imports
use serde;


#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct FixedSizeQueue<T, const L: usize>(pub VecDeque<T>);

// There are better ways to implement this, I know.
impl<T, const L: usize> FixedSizeQueue<T, L> {
    // returns an element if the queue is in full capacity
    pub fn push(&mut self, element: T) -> Option<T> {
        self.0.push_front(element);

        if self.0.len() > L {
            self.0.pop_back()
        } else {
            None
        }
    }
}

impl<T: PartialEq, const L: usize> FixedSizeQueue<T, L> {
    pub fn push_without_duplicates(&mut self, element: T) -> Option<T> {
        self.0.retain(|r| *r != element);
        self.push(element)
    }
}


impl App {
    pub fn update_emulator(&mut self) {
        let Some(ref mut nes) = self.nes else { return; };

        // clock nes until the ppu finishes drawing the frame
        while !nes.check_frame_complete_and_toggle() { nes.clock(&mut self.nes_screen.buffer) };
        self.nes_screen.update();

        // if we want to render pattern memory
        if self.app_settings.show.pattern_memory {
            let pattern_buffer = &mut self.pattern_screen.buffer;
            nes.render_debug_pattern_table(pattern_buffer, PatternTable::Left, self.debug_state.chosen_debug_palette);
            nes.render_debug_pattern_table(pattern_buffer, PatternTable::Right, self.debug_state.chosen_debug_palette);
            self.pattern_screen.update();
        }
        // match self.debug_state.running_mode {
        //     RunningMode::CyclesPerFrame(speed) => {
        //         for _ in 0..speed {
        //             nes.clock(&mut self.nes_screen.unwrap().buffer);

        //             if nes.check_frame_complete_and_toggle() {
        //                 screen.update();
        //                 if let (true, Some(pattern_screen)) = (self.app_settings.show.pattern_memory, self.pattern_screen.as_mut()) {
        //                     let pattern_buffer = &mut self.pattern_screen.unwrap().buffer;
        //                     nes.render_debug_pattern_table(pattern_buffer, PatternTable::Left, self.debug_state.chosen_debug_palette);
        //                     nes.render_debug_pattern_table(pattern_buffer, PatternTable::Right, self.debug_state.chosen_debug_palette);
        //                     pattern_screen.update();
        //                 }
        //             };
        //         };
        //     },
        //     RunningMode::StepByStep => {
        //         if nes.check_frame_complete_and_toggle() {
        //             screen.update();
                    
        //             if let (true, Some(pattern_screen)) = (self.app_settings.show.pattern_memory, self.pattern_screen.as_mut()) {
        //                 let pattern_buffer = &mut self.pattern_screen.unwrap().buffer;
        //                 nes.render_debug_pattern_table(pattern_buffer, PatternTable::Left, self.debug_state.chosen_debug_palette);
        //                 nes.render_debug_pattern_table(pattern_buffer, PatternTable::Right, self.debug_state.chosen_debug_palette);
        //                 pattern_screen.update();
        //             }
        //         };
        //     },
        //     RunningMode::Normal => {
        //         // clock nes until the ppu finishes drawing the frame
        //         while !nes.check_frame_complete_and_toggle() { nes.clock(&mut self.nes_screen.unwrap().buffer) };
        //         screen.update();

        //         // if we want to render pattern memory
        //         if let (true, Some(pattern_screen)) = (self.app_settings.show.pattern_memory, self.pattern_screen.as_mut()) {
        //             let pattern_buffer = &mut self.pattern_screen.unwrap().buffer;
        //             nes.render_debug_pattern_table(pattern_buffer, PatternTable::Left, self.debug_state.chosen_debug_palette);
        //             nes.render_debug_pattern_table(pattern_buffer, PatternTable::Right, self.debug_state.chosen_debug_palette);
        //             pattern_screen.update();
        //         }
        //     }
        // }
    }

    pub fn load_rom(&mut self, path: &PathBuf) -> Result<NES, ROMLoadError> {
        let file = INesFile::try_new_from_path(path)?;
        let cart = Cartridge::try_from(&file)?;

        // todo: this is not meant to be done here.
        // todo: remove this and change the parameter to be &self instead of &mut self
        self.instructions = Some(cart.disassemble());

        let nes = nes_backend::nes::NES::new(cart);
        Ok(nes)
    }
}

#[derive(Debug, Error)]
pub enum ROMLoadError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    FileDecoding(#[from] FileDecodingError),

    #[error(transparent)]
    UnimplementedMapper(#[from] UnimplementedMapperError),
}
