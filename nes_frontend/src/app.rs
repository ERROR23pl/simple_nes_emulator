mod ui;
mod utils;

use egui::Key;
use log::error;
// crate import
use nes_backend::{nes::NES, cpu::disassembler::*};
use utils::display::{Screen, BasicPixelBuffer};
// std imports
use std::path::PathBuf;

use crate::app::utils::ROMLoadError;

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
    nes: Option<NES<BasicPixelBuffer>>,
    
    // todo: instructions should be part of NES
    #[serde(skip)]
    instructions: Option<Vec<DisassembledInstruction>>,

    #[serde(skip)]
    events: Events,
    app_settings: AppSettings,
    debug_state: DebugState,
}

// * instantiation
impl App {
    pub fn default_with_context(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            nes_screen: Some(Screen::new(&cc.egui_ctx, "nes screen", 256, 240)),
            pattern_screen: Some(Screen::new(&cc.egui_ctx, "pattern screen", 256, 128)),
            nes: None,
            instructions: None,
            recent_roms: utils::FixedSizeQueue::default(),
            events: Events::default(),
            app_settings: AppSettings::default(),
            debug_state: DebugState::default(),
        }
    }

    // Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // todo: this is where we can bring back the old state after quitting.
        // todo: check `old_app.rs` to find out how.
        Self::default_with_context(cc)
    }
    
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

impl eframe::App for App {
    // Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Use in order for gui to be drawn even without user input.
        // The game has to be drawn even if the user does nothing.
        ctx.request_repaint();

        // ctx.input(|i| {
        //     for event in &i.events {
        //         println!("{event:?}");
        //     }
        // });
        
        if self.nes.is_some() {
            self.update_emulator();
        }

        // controller input
        let mapping = &self.app_settings.controller_mapping;
        self.debug_state.controller_state = ctx.input(|i| ControllerState {
            a: i.key_down(mapping.a),
            b: i.key_down(mapping.b),
            select: i.key_down(mapping.select),
            start: i.key_down(mapping.start),
            up: i.key_down(mapping.up),
            down: i.key_down(mapping.down),
            left: i.key_down(mapping.left),
            right: i.key_down(mapping.right),
        });

        self.controller_input(ctx);

        // let input_buttons = ctx.input(|i| [
        //     i.key_down(mapping.a),
        //     i.key_down(mapping.b),
        //     i.key_down(mapping.select),
        //     i.key_down(mapping.start),
        //     i.key_down(mapping.up),
        //     i.key_down(mapping.down),
        //     i.key_down(mapping.left),
        //     i.key_down(mapping.right),
        // ]);

        // todo: second controller
        if let Some(ref mut nes) = self.nes {
            let input_buttons = [
                self.debug_state.controller_state.a,
                self.debug_state.controller_state.b,
                self.debug_state.controller_state.select,
                self.debug_state.controller_state.start,
                self.debug_state.controller_state.up,
                self.debug_state.controller_state.down,
                self.debug_state.controller_state.left,
                self.debug_state.controller_state.right,
            ];

            let controlers_1 = &mut nes.cpu.controlers[0];

            *controlers_1 = 0x00;
            for i in input_buttons {
                *controlers_1 <<= 1;
                *controlers_1 |= i as u8;
            }
        }

        self.main_ui(ctx);        
    }
}
#[derive(Default)]
pub struct Events {
    file_loading_error_modal: Option<ROMLoadError>,
}

#[derive(Default, serde::Serialize)]
pub struct AppSettings {
    show: ShowSettings,
    controller_mapping: ControlerMapping,
}

#[derive(serde::Serialize)]
pub struct ShowSettings {
    pattern_memory: bool,
    cpu_state: bool,
    program_counter: bool,
    ram: bool,
}

impl Default for ShowSettings {
    fn default() -> Self {
        Self { pattern_memory: true, cpu_state: true, program_counter: true, ram: true }
    }
}

#[derive(serde::Serialize)]
pub struct DebugState {
    program_state_offset: usize,
    ram_page_number: usize,
    ram_editing: bool,
    chosen_debug_palette: u8,
    running_mode: RunningMode,
    controller_state: ControllerState,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            program_state_offset: 5,
            ram_page_number: 0,
            ram_editing: false,
            chosen_debug_palette: 0,
            running_mode: RunningMode::Normal,
            controller_state: ControllerState::default(),
        }
    }
}

#[derive(Default, serde::Serialize)]
pub enum RunningMode {
    #[default]
    Normal,
    StepByStep,
    CyclesPerFrame(u64),
}

#[derive(serde::Serialize)]
struct ControlerMapping {
    right: Key,
    left: Key,
    down: Key,
    up: Key,
    start: Key,
    select: Key,
    b: Key,
    a: Key,
}

impl Default for ControlerMapping {
    fn default() -> Self {
        Self {
            right: Key::L,
            left: Key::J,
            down: Key::K,
            up: Key::I,
            start: Key::S,
            select: Key::A,
            b: Key::Z,
            a: Key::X,
        }
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ControllerState {
    right: bool,
    left: bool,
    down: bool,
    up: bool,
    start: bool,
    select: bool,
    b: bool,
    a: bool,
}