use super::*;
use super::debug::DebugState;

#[derive(serde::Serialize)]
pub struct ControllerMapping {
    right: Key,
    left: Key,
    down: Key,
    up: Key,
    start: Key,
    select: Key,
    b: Key,
    a: Key,
}

impl Default for ControllerMapping {
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

impl ControllerMapping {
    pub fn standard_player_2() -> Self {
        Self {
            right: Key::ArrowRight,
            left: Key::ArrowLeft,
            down: Key::ArrowDown,
            up: Key::ArrowUp,
            start: Key::Enter,
            select: Key::Equals,
            b: Key::Q,
            a: Key::W,
        }
    }

    pub fn standard_mapping_both_players() -> [Self; 2] {
        [Self::default(), Self::standard_player_2()]
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ControllerState {
    pub right: bool,
    pub left: bool,
    pub down: bool,
    pub up: bool,
    pub start: bool,
    pub select: bool,
    pub b: bool,
    pub a: bool,
}

impl App {
    pub fn update_controllers(&mut self, ctx: &egui::Context) {
        for player in 0..=1 {
            self.save_controller_state(ctx, player);

            self.load_controller_state_into_nes(player);
        }
        
        self.controller_debug_ui(ctx);
    }

    fn save_controller_state(&mut self, ctx: &egui::Context, player: usize) {
        let mapping = &self.app_settings.controller_mapping[player];

        self.controller_state[player] = ctx.input(|i| ControllerState {
            a:      i.key_down(mapping.a),
            b:      i.key_down(mapping.b),
            select: i.key_down(mapping.select),
            start:  i.key_down(mapping.start),
            up:     i.key_down(mapping.up),
            down:   i.key_down(mapping.down),
            left:   i.key_down(mapping.left),
            right:  i.key_down(mapping.right),
        });
    }

    fn load_controller_state_into_nes(&mut self, player: usize) {
        let Some(ref mut nes) = self.nes else { return; };

        let controller_state = &self.controller_state[player];
        let input_buttons = [
            controller_state.a,
            controller_state.b,
            controller_state.select,
            controller_state.start,
            controller_state.up,
            controller_state.down,
            controller_state.left,
            controller_state.right,
        ];

        let controlers_1 = &mut nes.cpu.controlers[player];

        *controlers_1 = 0x00;
        for i in input_buttons {
            *controlers_1 <<= 1;
            *controlers_1 |= i as u8;
        }
    }
}
