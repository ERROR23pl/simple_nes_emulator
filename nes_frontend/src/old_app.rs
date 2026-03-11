mod utils;
mod image;
mod display;

use egui::{Color32, ColorImage, ImageOptions, TextureOptions};

use crate::app::image::{test_image, update_image};


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    recent_roms: utils::FixedQueue<String, 5>,

    color: [u8; 3],
    frame: usize,

    #[serde(skip)]
    screen: Option<egui::TextureHandle>,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}



impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Zmieniłem label lol :D".to_owned(),
            frame: 0,
            color: [0, 0, 0],
            screen: None,
            recent_roms: utils::FixedQueue::new(),
            value: 2.7,
        }
    }
}

impl App {
    pub fn default_with_context(cc: &eframe::CreationContext<'_>) -> Self {
        println!("this function has been called");
        Self {
            screen: Some(cc.egui_ctx.load_texture("screen", test_image(), TextureOptions::default())),
            ..Default::default()
        }
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        println!("Is this even called?");
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_else(|| Self::default_with_context(cc))
        //     Self::default_with_context(cc)
        // } else {
        //     Self::default_with_context(cc)
        // }
        Self::default_with_context(cc)
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                // let is_web = cfg!(target_arch = "wasm32");
                // if !is_web {
                //     ui.menu_button("File", |ui| {
                //         if ui.button("Quit").clicked() {
                //             ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                //         }
                //     });
                //     ui.add_space(16.0);
                // }
                
                ui.menu_button("File", |ui| {
                    // todo: implement loading roms
                    if ui.button("load ROM").clicked() {
                        println!("clicked load ROM");
                    }

                    ui.menu_button("recent ROMs", |ui| {
                        for (i, title) in self.recent_roms.0.clone().iter().enumerate() {
                            let _ = ui.button(format!("{}. {}", i+1, title.as_str()));
                        }
                    });
                });
                
                ui.menu_button("theme", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });

                // todo: add "are you sure?" prompt
                // #[cfg(not(target_arch = "wasm32"))]
                // ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                // if ui.button("Quit").clicked() {
                //     egui::Modal::new(egui::Id::new("quit_prompt")).show(ui.ctx(), |ui| {
                //         ui.heading("Are you sure you want to quit emulation?");
                //         ui.horizontal(|ui| {
                //             if ui.button("Yes").clicked() {
                //                 ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                //             }

                //             if ui.button("No").clicked() {
                //                 ui.close();
                //             }
                //         });
                //     });
                // }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
                if ui.button("Add ROM").clicked() {
                    self.recent_roms.push(self.label.clone());
                }
            });

            // ui.color_edit_button_srgba(&mut self.color);
            ui.color_edit_button_srgb(&mut self.color);
            update_image(self.color, self.screen.as_mut().unwrap(), &mut self.frame);
            
            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        
        egui::Window::new("NES display")
            .resizable(false)
            .id("nes_window".into())
            // .vscroll(false)
            .resizable(false)
            .min_height(240.0)
            .min_width(256.0)
            .show(ctx, |ui| {
                match self.screen.as_ref() {
                    Some(handle) => {
                        ui.add({
                            egui::Image::new(handle).fit_to_exact_size(ui.available_size())
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
                match self.screen.as_ref() {
                    Some(handle) => {
                        ui.add({
                            egui::Image::new(handle).fit_to_exact_size(ui.available_size())
                        });
                    }
                    None => { println!("No i chu") },
                }
            });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
