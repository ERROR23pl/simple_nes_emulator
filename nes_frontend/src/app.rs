mod utils;
mod screen;

// crate import
use crate::app::screen::{DummyCanvas, Screen};

// std imports
use std::path::PathBuf;

use egui::{FontId, InnerResponse};
use nes_backend::{
    cartridge::{self, Cartridge},
    file_loading,
    cpu::disassembler::*,
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
    nes: Option<nes_backend::nes::NES<DummyCanvas>>,
    
    #[serde(skip)]
    instructions: Option<Vec<DisassembledInstruction>>,
    
    // todo: delete all this shit
    test_color: [u8; 3],
    frame: usize,
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
        self.nes_screen.as_mut().unwrap().color_flicker(self.test_color, &mut self.frame);

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
                    None => { println!("No i chu") },
                }
            });

        egui::Window::new("Pattern Memory")
            .resizable(false)
            .id("pattern_screen".into())
            // .vscroll(false)
            .resizable(true)
            // .min_height(128.0)
            // .min_width(256.0)
            .show(ctx, |ui| {
                match self.pattern_screen.as_ref() {
                    Some(screen) => {
                        ui.add({
                            egui::Image::new(&screen.handle).fit_to_exact_size(ui.available_size())
                        });
                    }
                    None => { println!("No i chuj") },
                }
            });
        
        egui::Window::new("Program memory")
            .resizable(false)
            .id("program_memory_screen".into())
            // .vscroll(false)
            .resizable(true)
            // .min_height(128.0)
            // .min_width(256.0)
            .show(ctx, |ui| {
                let Some(ref mut nes) = self.nes else { return; };
                let program_counter = nes.cpu().program_counter() as usize;
                
                let Some(ref instructions) = self.instructions else { return; };
                
                if ui.button(format!("clock CPU")).clicked() {
                    for _ in 0..3 { nes.clock() };
                }
                
                if ui.button(format!("next instruction")).clicked() {
                    while nes.cpu().cycles() != 0 { nes.clock() };
                    for _ in 0..3 { nes.clock() };
                }
                
                let cpu = nes.cpu();
                ui.label(format!("acc: {:02X}    x: {:02X}    y: {:02X}", cpu.acc(), cpu.reg_x(), cpu.reg_y()));
                ui.label(format!("pc: {:04X}    stack: {:02X}    status: {:02b}", cpu.stack_pointer(), cpu.program_counter(), cpu.status()));
                ui.label(format!("cycles: {}", cpu.cycles()));
                
                // let binary_index = instructions.binary_search_by(|i| i.address().cmp(&program_counter));
                let Some(index) = instructions.iter().position(|i| i.address() == program_counter) else { return; };


                const INSTRUCTION_OFFSET: usize = 10;
                let instructions_slice = &instructions[
                    index.saturating_sub(INSTRUCTION_OFFSET)..=index.saturating_add(INSTRUCTION_OFFSET)
                ];    

                for instr in instructions_slice {
                    let is_current_instruction = instr.address() == program_counter;
                    let text = egui::RichText::new(format!("{}{}", if is_current_instruction { "> " } else { "  " }, instr))
                        .font(FontId::monospace(16.0));

                    if is_current_instruction {
                        ui.label(text.background_color(egui::Color32::from_rgb(48, 48, 48))).highlight();
                    } else {
                        ui.label(text);
                    }
                }

                // todo: show here the debug info about prg_rom
            });
    }
}

impl App {
    fn top_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // todo: implement loading roms
                    if ui.button("load ROM").clicked() {
                        let Some(selected_rom_path) = FileDialog::new()
                            .add_filter("text", &["ines", "nes"])
                            .set_directory("/")
                            .pick_file()
                        else { return; };

                        let _ = self.recent_roms.push_without_duplicates(selected_rom_path.clone());
                        self.load_rom(&selected_rom_path);
                        // todo: select the rom and play the game 
                    }
                    
                    ui.menu_button("recent ROMs", |ui| {
                        for (i, title) in self.recent_roms.0.iter().enumerate() {
                            ui.set_min_width(300.0);
                            let _ = ui.button(format!("{}. {}", i+1, title.file_name().unwrap().to_str().unwrap()));
                            ui.set_min_width(0.0);
                        }
                    });
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
                
        self.instructions = Some(file.disassemble());
        self.nes = Some(nes_backend::nes::NES::new(DummyCanvas, cart));
    }
}