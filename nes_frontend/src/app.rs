mod ui;
mod utils;

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

#[derive(Default)]
pub struct Events {
    file_loading_error_modal: Option<ROMLoadError>,
}

#[derive(Default, serde::Serialize)]
pub struct AppSettings {
    show: ShowSettings,
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

