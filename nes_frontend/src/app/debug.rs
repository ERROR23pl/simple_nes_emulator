#[derive(serde::Serialize)]
pub struct DebugState {
    pub program_state_offset: usize,
    pub ram_page_number: usize,
    pub ram_editing: bool,
    pub chosen_debug_palette: u8,
    pub running_mode: RunningMode,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            program_state_offset: 5,
            ram_page_number: 0,
            ram_editing: false,
            chosen_debug_palette: 0,
            running_mode: RunningMode::Normal,
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


