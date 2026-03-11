mod utils;
mod image;
mod display;
mod screen;

// std imports
use std::path::PathBuf;

// crate.io imports
use rfd::FileDialog;


use crate::app::screen::Screen;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // stores paths to the last 5 ROMs loaded 
    recent_roms: utils::FixedSizeQueue<PathBuf, 5>,

    #[serde(skip)]
    nes_screen: Option<Screen>,

    #[serde(skip)]
    pattern_screen: Option<Screen>,
    
    // todo: delete all this shit
    test_color: [u8; 3],
    frame: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            frame: 0,
            test_color: [0, 0, 0],
            nes_screen: None,
            pattern_screen: None,
            recent_roms: utils::FixedSizeQueue::default(),
        }
    }
}

impl App {
    pub fn default_with_context(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            // nes_screen: Some(cc.egui_ctx.load_texture("screen", test_image(), egui::TextureOptions::default())),
            nes_screen: Some(Screen::new(&cc.egui_ctx, "nes screen", 256, 240)),
            // pattern_screen: Some(cc.egui_ctx.load_texture("screen", test_image(), egui::TextureOptions::default())),
            pattern_screen: Some(Screen::new(&cc.egui_ctx, "pattern screen", 256, 128)),
            ..Default::default()
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
        // self.nes_screen.as_mut().unwrap().update_image(self.test_color, &mut self.frame);

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
                        let selected_rom_path = FileDialog::new()
                            .add_filter("text", &["ines", "nes"])
                            .set_directory("/")
                            .pick_file();

                        match selected_rom_path {
                           Some(p) => { let _ = self.recent_roms.push(p); },
                           None => { },
                        };
                        // todo: select the rom and play the game 
                    }
                    
                    ui.menu_button("recent ROMs", |ui| {
                        // todo: make these buttons wider
                        for (i, title) in self.recent_roms.0.clone().iter().enumerate() {
                            let _ = ui.button(format!("{}. {}", i+1, title.file_name().unwrap().to_str().unwrap()));
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
