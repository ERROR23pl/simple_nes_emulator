mod debug_screens;

// crate imports
use crate::App;

// third-party imports
use egui::{Id, Image, Modal};
use rfd::FileDialog;

impl eframe::App for App {
    // Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Use in order for gui to be drawn even without user input.
        // The game has to be drawn even if the user does nothing.
        ctx.request_repaint();
        
        self.top_panel(ctx);

        if self.nes.is_some() {
            self.update_emulator();
            self.nes_display(ctx);
            self.debug_windows(ctx);
        }
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

                    let Some(rom) = selected_rom_path else { return; };

                    match self.load_rom(&rom) {
                        Ok(nes) => {
                            self.nes = Some(nes);
                            let _ = self.recent_roms.push_without_duplicates(rom);
                        },
                        Err(error) => {
                            self.events.file_loading_error_modal = Some(error);
                        },
                    }
                });
                
                if let Some(error) = self.events.file_loading_error_modal.take() {
                    Modal::new(Id::new("file_loading_error_modal")).show(ui.ctx(), |ui| {
                        ui.label(format!("{}", error));
                        self.events.file_loading_error_modal = (!ui.button("Ok").clicked()).then_some(error);
                    });
                }
                
                ui.menu_button("theme", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        })
    }
    
    fn nes_display(&mut self, ctx: &egui::Context) {
        egui::Window::new("NES display")
            .resizable(false)
            .id("nes_window".into())
            .min_width(256.0)
            .min_height(240.0)
            .show(ctx, |ui| {
                let Some(screen) = self.nes_screen.as_ref() else { return; };
                ui.add(Image::new(&screen.handle)
                    .fit_to_exact_size(ui.available_size())
                );
            });
    }
}
