use crate::App;
use crate::app::controller::ControlerMapping;


#[derive(Default, serde::Serialize)]
pub struct AppSettings {
    show: ShowSettings,
    pub controller_mapping: [ControlerMapping; 2],
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