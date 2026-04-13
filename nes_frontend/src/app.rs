mod utils;
mod screen;
mod debug_screens;

// crate import
use crate::app::screen::{BasicPixelBuffer, DummyCanvas, Screen};

// std imports
use std::path::PathBuf;

use egui::{Color32, FontId, InnerResponse, Rgba, RichText};
use nes_backend::{
    cartridge::{self, Cartridge}, cpu::{Status, disassembler::*}, file_loading, rendering::{PAL_COLOR, PatternTable}
};
// crate.io imports
use rfd::FileDialog;


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(/*serde::Deserialize,*/ serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // stores paths to the last 5 ROMs loaded 
    recent_roms: utils::FixedSizeQueue<PathBuf, 5>,

    #[serde(skip)]
    nes_screen: Option<Screen>,

    #[serde(skip)]
    pattern_screen: Option<Screen>,

    #[serde(skip)]
    nes: Option<nes_backend::nes::NES<BasicPixelBuffer>>,
    
    #[serde(skip)]
    instructions: Option<Vec<DisassembledInstruction>>,

    debug_state: DebugState,
    // todo: delete all this shit
    test_color: [u8; 3],
    frame: usize,
}

pub struct AppSettings {
    show: ShowSettings,
}

pub struct ShowSettings {
    pattern_memory: bool,
    cpu_state: bool,
    program_counter: bool,
    ram: bool,
}

#[derive(serde::Serialize)]
pub struct DebugState {
    program_state_offset: usize,
    ram_page_number: usize,
    ram_editing: bool,
    chosen_debug_palette: u8,
    run_speed: Option<i32>,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            program_state_offset: 5,
            ram_page_number: 0,
            ram_editing: false,
            chosen_debug_palette: 0,
            run_speed: None,
        }
    }
}

// impl Default for App {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             frame: 0,
//             test_color: [0, 0, 0],
//             nes: None,
//             nes_screen: None,
//             pattern_screen: None,
//             recent_roms: utils::FixedSizeQueue::default(),
//         }
//     }
// }

impl App {
    pub fn default_with_context(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            nes_screen: Some(Screen::new(&cc.egui_ctx, "nes screen", 256, 240)),
            pattern_screen: Some(Screen::new(&cc.egui_ctx, "pattern screen", 256, 128)),
            frame: 0,
            test_color: [0, 0, 0],
            nes: None,
            instructions: None,
            recent_roms: utils::FixedSizeQueue::default(),
            debug_state: DebugState::default(),
        }
    }

    // Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // todo: this is where we can bring back the old state after quitting.
        // todo: check `old_app.rs` to find out how.
        Self::default_with_context(cc)
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    // Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Use in order for gui to be drawn even without user input.
        // The game has to be drawn even if the user does nothing.
        ctx.request_repaint();
        // self.nes_screen.as_mut().unwrap().color_flicker(self.test_color, &mut self.frame);

        if let (Some(nes), Some(_)) = (&mut self.nes, self.debug_state.run_speed) {
            nes.clock();
        }
      

        self.top_panel(ctx);

        egui::Window::new("NES display")
            .resizable(false)
            .id("nes_window".into())
            // .vscroll(false)
            .resizable(true)
            .min_width(256.0)
            .min_height(240.0)
            .show(ctx, |ui| {
                match self.nes_screen.as_ref() {
                    Some(screen) => {
                        ui.add({
                            egui::Image::new(&screen.handle).fit_to_exact_size(ui.available_size())
                        });
                    }
                    None => { ui.label("unable to render the debug screen. Something went wrong."); },
                }
            });

        let Some(ref mut pattern_screen) = self.pattern_screen else { return; };
        if let Some(ref mut nes) = self.nes {
            nes.get_mut_ppu().render_debug_pattern_table(PatternTable::Left, 1);
            nes.get_mut_ppu().render_debug_pattern_table(PatternTable::Right, 1);
        }
 
        egui::Window::new("Pattern Memory")
            .resizable(false)
            .id("pattern_screen".into())
            // .vscroll(false)
            .resizable(false)
            .min_height(128.0)
            .min_width(256.0)
            .show(ctx, |ui| {
                // ui.horizontal(|ui| {
                //     for i in 0..=7 {
                //         ui.radio_value(&mut self.debug_state.chosen_debug_palette, i, format!("#{}", i));
                //         let Some(ref nes) = self.nes else { return; };
                //         for j in 0..=3 {
                //             ui.label(RichText::from("■").color({
                //                 let nes_color = nes.ppu().get_color_value_from_pallet_ram(i, j);
                //                 let [r, g, b] = PAL_COLOR[nes_color as usize];
                //                 Rgba::from_rgb(r as f32, g as f32, b as f32)
                //             }));
                //         }
                //     }
                // });
                match self.pattern_screen.as_ref() {
                    Some(screen) => {
                        ui.add({
                            egui::Image::new(&screen.handle).fit_to_exact_size(ui.available_size())
                        });
                    }
                    None => { println!("No i chuj") },
                }
            });
        
        self.debug_windows(ctx);
    }
}

impl App {
    fn top_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar
            egui::MenuBar::new().ui(ui, |ui| {
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

                    if let Some(rom) = selected_rom_path {
                        self.load_rom(&rom);
                        let _ = self.recent_roms.push_without_duplicates(rom);
                    }
                });
                
                ui.menu_button("theme", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
                
                // todo: add quitting funcitonality with an "are you sure?" prompt and not on wasm32
                // todo: look at `old_app.rs` for inspiration.
            });
        })
    }
}

// * functional parts
impl App {
    fn load_rom(&mut self, path: &PathBuf) {
        let Ok(file) = file_loading::INesFile::new_from_file_path(path) else { return; };

        let Ok(cart) = cartridge::Cartridge::new(&file) else { return; };

        // todo: change these returns to error messages
        // todo: change the implementations so that errors are thrown in the backend and handled here with a single match statement
                
        self.instructions = Some(cart.disassemble());
        self.nes = Some(nes_backend::nes::NES::new(BasicPixelBuffer::new(256, 240), BasicPixelBuffer::new(256, 128), cart));
    }
}