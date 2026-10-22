
// crate imports
use super::App;
use super::settings::AppSettings;

// third-party imports
use egui::Key;

#[derive(serde::Serialize)]
pub struct ControlerMapping {
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


impl App {
    // todo: actually transfer this data to the app
    pub fn read_controller_input(&mut self) {
        let mapping = &self.app_settings.controller_mapping;
        

        // self.debug_state.controller_state = ctx.input(|i| ControllerState {
        //     a: i.key_down(mapping.a),
        //     b: i.key_down(mapping.b),
        //     select: i.key_down(mapping.select),
        //     start: i.key_down(mapping.start),
        //     up: i.key_down(mapping.up),
        //     down: i.key_down(mapping.down),
        //     left: i.key_down(mapping.left),
        //     right: i.key_down(mapping.right),
        // });
    }
}