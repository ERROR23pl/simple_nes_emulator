use egui::{Color32, DragValue, FontId, Image, Pos2, Rect, Rgba, RichText, Stroke, Ui, Vec2};
use nes_backend::{cpu::{CPU_RAM_SIZE, StatusFlag}, rendering::PAL_COLOR};

impl crate::App {
    pub fn debug_windows(&mut self, ctx: &egui::Context) {
        self.pattern_screen(ctx);
        self.cpu_state(ctx);
        self.program_counter_state(ctx);
        self.ram_state(ctx);
    }

    fn pattern_screen(&mut self, ctx: &egui::Context) {
        if !self.app_settings.show.pattern_memory { return; }
        let nes = self.nes.as_mut().expect("I've already checked if it's Some(_) on the call site.");

        egui::Window::new("Pattern Memory")
            .resizable(true)
            .id("pattern_window".into())
            .min_height(128.0)
            .min_width(256.0)
            .show(ctx, |ui| {
                let mut generate_palletes = |start, end, ui: &mut egui::Ui| {
                    for pallette_id in start..=end {
                        // palette picker
                        ui.radio_value(
                            &mut self.debug_state.chosen_debug_palette,
                            pallette_id,
                            format!("#{}", pallette_id)
                        );

                        // palette visualisation
                        for color_index in 0..=3 {
                            ui.label(RichText::from("■").color({
                                let nes_color = nes.cpu.ppu.get_color_value_from_pallet_ram(pallette_id, color_index);
                                let [r, g, b] = PAL_COLOR[nes_color as usize];
                                Rgba::from_rgb(r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32)
                            }));
                        }
                    }
                };

                ui.horizontal(|ui| generate_palletes(0, 3, ui));
                ui.horizontal(|ui| generate_palletes(4, 7, ui));
                
                let Some(screen) = self.pattern_screen.as_ref() else { return; };
                ui.add(Image::new(&screen.handle)
                    .fit_to_exact_size(ui.available_size())
                );
            });
    }

    fn cpu_state(&mut self, ctx: &egui::Context) {
        if !self.app_settings.show.cpu_state { return; }
        
        let nes = self.nes.as_mut().expect("I've already checked if it's Some(_) on the call site.");

        egui::Window::new("CPU State")
            .resizable(false)
            .id("cpu_state".into())
            .resizable(true)
            .show(ctx, |ui| {
                let cpu = nes.cpu();
                monospace_label(ui, format!("acc: {:02X}    x: {:02X}    y: {:02X}", cpu.acc(), cpu.reg_x(), cpu.reg_y()));
                monospace_label(ui, format!("pc: {:04X}    stack: {:02X}", cpu.program_counter(), cpu.stack_pointer()));
                monospace_label(ui, format!("cycles: {}", cpu.cycles()));
                monospace_label(ui, format!(
                    "Status:\n{:<20}{}\n{:<20}{}\n{:<20}{}\n{:<20}{}\n{:<20}{}\n{:<20}{}\n{:<20}{}\n{:<20}{}\n",
                    "  carry:", cpu.get_flag(StatusFlag::Carry),
                    "  zero:", cpu.get_flag(StatusFlag::Zero),
                    "  disableinterupts:", cpu.get_flag(StatusFlag::DisableInterupts),
                    "  decimalmode:", cpu.get_flag(StatusFlag::DecimalMode),
                    "  break:", cpu.get_flag(StatusFlag::Break),
                    "  unused:", cpu.get_flag(StatusFlag::Unused),
                    "  overflow:", cpu.get_flag(StatusFlag::Overflow),
                    "  negative:", cpu.get_flag(StatusFlag::Negative),
                ));
            });
    }

    fn program_counter_state(&mut self, ctx: &egui::Context) {
        if !self.app_settings.show.program_counter { return; }
        
        let nes = self.nes.as_mut().expect("I've already checked if it's Some(_) on the call site.");
        
        egui::Window::new("program state")
            .resizable(false)
            .id("program_state".into())
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(format!("nes cycle count: {}", nes.clock_count()));
                ui.label(format!("frame count: {}", nes.frame_count()));

                let program_counter = nes.cpu().program_counter() as usize;

                match self.debug_state.run_speed {
                    Some(ref mut speed) => {
                        ui.add(egui::DragValue::new(speed));
                        if ui.button("pause").clicked() {
                            self.debug_state.run_speed = None;
                        }
                    },
                    None => {
                        if ui.button("clock NES").clicked() {
                            nes.clock();
                        }

                        if ui.button("clock CPU").clicked() {
                            for _ in 0..3 { nes.clock() };
                        }
                        
                        if ui.button("next instruction").clicked() {
                            while nes.cpu().cycles() != 0 { nes.clock() };
                            for _ in 0..3 { nes.clock() };
                        }
                        
                        if ui.button("run emulation").clicked() {
                            self.debug_state.run_speed = Some(0);
                        }
                    }
                }

                let Some(ref instructions) = self.instructions else { return; };
                
                // ? why doesn't the following work? 
                // ? let binary_index = instructions.binary_search_by(|i| i.address().cmp(&program_counter));

                // todo: I actually don't remember why we have to go through the mapper. I just remember it works.
                let mapped_pc = nes.cpu().cartridge().mapper().map_cpu_read(program_counter as u16);
                let Some(index) = instructions.iter()
                    .position(|i|  mapped_pc as usize == i.address()) else { return; };


                let instruction_offset = self.debug_state.program_state_offset;
                let instructions_slice = &instructions[
                    index.saturating_sub(instruction_offset)..=index.saturating_add(instruction_offset)
                ];    
                
                ui.horizontal(|ui| {
                    ui.label("program offset: ");
                    ui.add(DragValue::new(&mut self.debug_state.program_state_offset));
                });
                
                for instr in instructions_slice {
                    let is_current_instruction = instr.address() == (mapped_pc as usize);
                    let text = RichText::new(format!("{}{}", if is_current_instruction { "> " } else { "  " }, instr))
                        .font(FontId::monospace(14.0));

                    if is_current_instruction {
                        ui.label(text.background_color(egui::Color32::from_rgb(48, 48, 48))).highlight();
                    } else {
                        ui.label(text);
                    }
                }
        });
    }

    fn ram_state(&mut self, ctx: &egui::Context) {
        if !self.app_settings.show.ram { return; }
        
        let nes = self.nes.as_mut().expect("I've already checked if it's Some(_) on the call site.");
        
        egui::Window::new("RAM state")
            .resizable(false)
            .id("ram_state".into())
            .resizable(true)
            .show(ctx, |ui| {
                const PAGE_SIZE: usize = 256;
                const NUMBER_OF_PAGES: usize = CPU_RAM_SIZE / PAGE_SIZE;

                let ram = nes.get_mut_cpu_ram();
                let current_page = self.debug_state.ram_page_number;
                
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("-1").clicked() {
                            self.debug_state.ram_page_number = current_page.saturating_sub(1);
                        }
                        
                        ui.label(format!("Page #{}", current_page));
                        
                        if ui.button("+1").clicked() {
                            self.debug_state.ram_page_number = std::cmp::min(NUMBER_OF_PAGES - 1, current_page + 1);
                        }
                    })
                });

                ui.checkbox(&mut self.debug_state.ram_editing, "edit RAM");

                egui::Grid::new("ram_state_grid")
                    .num_columns(17)
                    .min_col_width(0.0)
                    .spacing([0.0, 0.0])
                    .show(ui, |ui| {
                        ui.label(""); // empty to align the rest of labels
                        for column in 0..16 {
                            ui.label(egui::RichText::new(format!(" {:02X}", column)).monospace().strong());
                        }
                        ui.end_row();

                        for row in 0..16 {
                            ui.label(egui::RichText::new(
                                format!("${:04X}", current_page * PAGE_SIZE + row * 16)
                            ).monospace().strong());

                            for column in 0..16 {
                                let cell_index = current_page * PAGE_SIZE + row * 16 + column;
                                
                                if self.debug_state.ram_editing {
                                    ui.add(egui::DragValue::new(&mut ram[cell_index])
                                        .range(0..=255)
                                        .hexadecimal(2, false, true)
                                    );
                                } else {
                                    ui.monospace(format!(" {:02X}", ram[cell_index]));
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    pub fn controller_input(&self, ctx: &egui::Context) {
        egui::Window::new("controller_debug").show(ctx, |ui| {
            if let Some(ref nes) = self.nes {
                ui.label(format!("instant: {:?}", nes.cpu.controlers()));
                ui.label(format!("snapshot: {:?}", nes.cpu.controlers_state()));
            }
            let desired_size = Vec2::new(400.0, 200.0);
            let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

            let painter = ui.painter_at(rect);

            let grey = Color32::from_gray(100);
            let red = Color32::RED;

            let button_color = |pressed: bool| {
                if pressed {
                    red
                } else {
                    grey
                }
            };

            // =========================
            // D-Pad
            // =========================

            let dpad_center = Pos2::new(rect.left() + 70.0, rect.center().y);

            let dpad_size = 24.0;
            let gap = 2.0;

            // Up
            painter.rect_filled(
                Rect::from_center_size(
                    Pos2::new(dpad_center.x, dpad_center.y - dpad_size - gap),
                    Vec2::splat(dpad_size),
                ),
                2.0,
                button_color(self.debug_state.controller_state.up),
            );

            // Down
            painter.rect_filled(
                Rect::from_center_size(
                    Pos2::new(dpad_center.x, dpad_center.y + dpad_size + gap),
                    Vec2::splat(dpad_size),
                ),
                2.0,
                button_color(self.debug_state.controller_state.down),
            );

            // Left
            painter.rect_filled(
                Rect::from_center_size(
                    Pos2::new(dpad_center.x - dpad_size - gap, dpad_center.y),
                    Vec2::splat(dpad_size),
                ),
                2.0,
                button_color(self.debug_state.controller_state.left),
            );

            // Right
            painter.rect_filled(
                Rect::from_center_size(
                    Pos2::new(dpad_center.x + dpad_size + gap, dpad_center.y),
                    Vec2::splat(dpad_size),
                ),
                2.0,
                button_color(self.debug_state.controller_state.right),
            );

            // =========================
            // Start / Select
            // =========================

            let middle_y = rect.center().y + 50.0;

            let select_rect = Rect::from_center_size(
                Pos2::new(rect.center().x - 30.0, middle_y),
                Vec2::new(40.0, 12.0),
            );

            let start_rect = Rect::from_center_size(
                Pos2::new(rect.center().x + 30.0, middle_y),
                Vec2::new(40.0, 12.0),
            );

            painter.rect_filled(
                select_rect,
                4.0,
                button_color(self.debug_state.controller_state.select),
            );

            painter.rect_filled(
                start_rect,
                4.0,
                button_color(self.debug_state.controller_state.start),
            );

            // =========================
            // A / B buttons
            // =========================

            let button_center = Pos2::new(rect.right() - 80.0, rect.center().y);

            painter.circle_filled(
                Pos2::new(button_center.x - 25.0, button_center.y),
                18.0,
                button_color(self.debug_state.controller_state.b),
            );

            painter.circle_filled(
                Pos2::new(button_center.x + 25.0, button_center.y),
                18.0,
                button_color(self.debug_state.controller_state.a),
            );

            // Optional outlines
            painter.circle_stroke(
                Pos2::new(button_center.x - 25.0, button_center.y),
                18.0,
                Stroke::new(1.0, Color32::BLACK),
            );

            painter.circle_stroke(
                Pos2::new(button_center.x + 25.0, button_center.y),
                18.0,
                Stroke::new(1.0, Color32::BLACK),
            );
        });
    }
}

fn monospace_label(ui: &mut Ui, text: String) {
    ui.label(egui::RichText::new(text).font(FontId::monospace(16.0)));
}