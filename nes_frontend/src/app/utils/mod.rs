// module declarations
pub mod display;

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
        let Some(screen) = self.nes_screen.as_mut() else {
            return;
        };

        let nes = self.nes.as_mut().expect("I've already checked if it's Some(_) on the call site.");

        match self.debug_state.run_speed {
            Some(speed) => {
                for _ in 0..speed {
                    nes.clock()
                };
            },
            None => {
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

    pub fn load_rom(&mut self, path: &PathBuf) -> Result<NES<BasicPixelBuffer>, ROMLoadError> {
        let file = INesFile::new_from_file_path(path)?;
        let cart = Cartridge::try_from(&file)?;

        // todo: this is not meant to be done here.
        // todo: remove this and change the parameter to be &self instead of &mut self
        self.instructions = Some(cart.disassemble());

        let nes = nes_backend::nes::NES::new(BasicPixelBuffer::new(256, 240), BasicPixelBuffer::new(256, 128), cart);
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
