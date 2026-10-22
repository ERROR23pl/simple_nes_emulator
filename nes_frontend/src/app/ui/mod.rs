// module declaration
//// mod debug_screens;
mod file_loading;
mod display;

// re-imports to higher modules
pub use {
    file_loading::*,
    display::*,
};

// crate imports
use crate::App;

// third-party imports
use egui::{Id, Image, Modal};




impl App {
    pub fn ui(&mut self, ctx: &egui::Context) {
        self.top_panel(ctx);

        // if self.nes.is_some() {
        //     self.nes_display(ctx);
        //     self.debug_windows(ctx);
        // }
    }
    
    fn top_panel(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                self.file_button(ui);
                
                // todo: this will be implemented later with context
                // if let Some(error) = self.events.file_loading_error_modal.take() {
                //     Modal::new(Id::new("file_loading_error_modal")).show(ui.ctx(), |ui| {
                //         ui.label(format!("{}", error));
                //         self.events.file_loading_error_modal = (!ui.button("Ok").clicked()).then_some(error);
                //     });
                // }
                
                ui.menu_button("theme", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        })
    }
    
    // fn nes_display(&mut self, ctx: &egui::Context) {
    //     egui::Window::new("NES display")
    //         .resizable(false)
    //         .id("nes_window".into())
    //         .min_width(256.0)
    //         .min_height(240.0)
    //         .show(ctx, |ui| {
    //             let Some(screen) = self.nes_screen.as_ref() else { return; };
    //             ui.add(Image::new(&screen.handle)
    //                 .fit_to_exact_size(ui.available_size())
    //             );
    //         });
    // }
}
