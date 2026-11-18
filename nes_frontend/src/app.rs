mod ui;
mod utils;
mod controller;
mod debug;

// crate imports
use crate::app::utils::ROMLoadError;
use controller::*;
use debug::*;
use nes_backend::{nes::NES, cpu::disassembler::*};
use utils::display::{Screen, BasicPixelBuffer};

// std imports
use std::path::PathBuf;

// third-party imports
use log::error;
use egui::Key;


// ---------------------------------------------------------------------------------------------------- \

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(/*serde::Deserialize,*/ serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // stores paths to the last 5 ROMs loaded 
    recent_roms: utils::FixedSizeQueue<PathBuf, 5>,

    #[serde(skip)]
    nes_screen: Screen,

    #[serde(skip)]
    pattern_screen: Screen,

    #[serde(skip)]
    nes: Option<NES>,

    #[serde(skip)]
    controller_state: [ControllerState; 2],
    
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
            nes_screen: Screen::new(&cc.egui_ctx, "nes screen", 256, 240),
            pattern_screen: Screen::new(&cc.egui_ctx, "pattern screen", 256, 128),
            nes: None,
            controller_state: Default::default(),

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
        // Force the UI to redraw. By default it does so every 1/60th of a second.
        ctx.request_repaint();

        self.update_emulator();

        self.update_controllers(ctx);

        self.main_ui(ctx);        
    }
}

#[derive(Default)]
pub struct Events {
    file_loading_error_modal: Option<ROMLoadError>,
}

#[derive(serde::Serialize)]
pub struct AppSettings {
    show: ShowSettings,
    controller_mapping: [ControllerMapping; 2],
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            show: Default::default(),
            controller_mapping: ControllerMapping::standard_mapping_both_players()
        }
    }
}

#[derive(serde::Serialize)]
pub struct ShowSettings {
    pattern_memory: bool,
    cpu_state: bool,
    program_counter: bool,
    ram: bool,
    oam: bool,
}

impl Default for ShowSettings {
    fn default() -> Self {
        Self { pattern_memory: true, cpu_state: true, program_counter: true, ram: true, oam: true }
    }
}

