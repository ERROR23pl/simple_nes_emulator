// crate imports
use crate::App;

// std imports
use std::{collections::VecDeque, path::PathBuf};

// third-party imports
use rfd::FileDialog;
use serde;

impl App {
    pub fn file_button(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            let mut selected_rom_path = None;
            
            if ui.button("load ROM").clicked() {
                selected_rom_path = FileDialog::new()
                    .add_filter("text", &["ines", "nes"])
                    .set_directory("/")
                    .pick_file();
            }
            
            ui.menu_button("recent ROMs", |ui| {
                for (i, rom_path) in self.recent_roms.0.iter().enumerate() {
                    let title = rom_path.file_name().unwrap().to_str().unwrap();
                    ui.set_min_width(300.0);
                    if ui.button(format!("{}. {}", i+1, title)).clicked() {
                        selected_rom_path = Some(rom_path.clone());
                    }
                    ui.set_min_width(0.0);
                }
            });

            let Some(rom) = selected_rom_path else { return; };

            // match self.load_rom(&rom) {
            //     Ok(nes) => {
            //         self.nes = Some(nes);
            //         let _ = self.recent_roms.push_without_duplicates(rom);
            //     },
            //     Err(error) => {
            // todo: this should just use context variables
            //         self.events.file_loading_error_modal = Some(error);
            //     },
            // }
        });
    }

    // pub fn load_rom(&mut self, path: &PathBuf) -> Result<NES<BasicPixelBuffer>, ROMLoadError> {
    //     let file = INesFile::try_new_from_path(path)?;
    //     let cart = Cartridge::try_from(&file)?;

    //     // todo: this is not meant to be done here.
    //     // todo: remove this and change the parameter to be &self instead of &mut self
    //     self.instructions = Some(cart.disassemble());

    //     let nes = nes_backend::nes::NES::new(BasicPixelBuffer::new(256, 240), BasicPixelBuffer::new(256, 128), cart);
    //     Ok(nes)
    // }
}

// #[derive(Debug, Error)]
// pub enum ROMLoadError {
//     #[error(transparent)]
//     IO(#[from] io::Error),

//     #[error(transparent)]
//     FileDecoding(#[from] FileDecodingError),

//     #[error(transparent)]
//     UnimplementedMapper(#[from] UnimplementedMapperError),
// }


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