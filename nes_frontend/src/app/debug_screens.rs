use std::cell;

use egui::{FontId, Ui};
use nes_backend::cpu::StatusFlag;

impl crate::App {
    pub fn debug_windows(&mut self, ctx: &egui::Context) {
        self.cpu_state(ctx);
        self.program_counter_state(ctx);
        self.ram_state(ctx);
    }

    pub fn cpu_state(&mut self, ctx: &egui::Context) {
        egui::Window::new("CPU State")
            .resizable(false)
            .id("CPU state".into())
            .resizable(true)
            .show(ctx, |ui| {
                let Some(ref mut nes) = self.nes else { return; };
                
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

    pub fn program_counter_state(&mut self, ctx: &egui::Context) {
        egui::Window::new("program state")
            .resizable(false)
            .id("program state".into())
            .resizable(true)
            .show(ctx, |ui| {
                let Some(ref mut nes) = self.nes else { return; };
                ui.label(format!("nes cycle count: {}", nes.clock_count()));

                let program_counter = nes.cpu().program_counter() as usize;
                let Some(ref instructions) = self.instructions else { return; };

                match self.debug_state.run_speed {
                    Some(ref mut speed) => {
                        ui.add(egui::DragValue::new(speed));
                        if ui.button("pause").clicked() {
                            self.debug_state.run_speed = None;
                        }
                    },
                    None => {
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
                

                // let binary_index = instructions.binary_search_by(|i| i.address().cmp(&program_counter));
                let mapped_pc = nes.cpu().bus().borrow().get_cartridge().mapper().map_cpu_read(program_counter as u16);
                let Some(index) = instructions.iter()
                    .position(|i|  mapped_pc as usize == i.address()) else { return; };


                let instruction_offset = self.debug_state.program_state_offset;
                let instructions_slice = &instructions[
                    index.saturating_sub(instruction_offset)..=index.saturating_add(instruction_offset)
                ];    
                
                ui.horizontal(|ui| {
                    ui.label("program offset: ");
                    ui.add(egui::DragValue::new(&mut self.debug_state.program_state_offset));
                });
                
                for instr in instructions_slice {
                    let is_current_instruction = instr.address() == (mapped_pc as usize);
                    let text = egui::RichText::new(format!("{}{}", if is_current_instruction { "> " } else { "  " }, instr))
                        .font(FontId::monospace(14.0));

                    if is_current_instruction {
                        ui.label(text.background_color(egui::Color32::from_rgb(48, 48, 48))).highlight();
                    } else {
                        ui.label(text);
                    }
                }
        });
    }

    pub fn ram_state(&mut self, ctx: &egui::Context) {
        egui::Window::new("RAM state")
            .resizable(false)
            .id("RAM state".into())
            .resizable(true)
            .show(ctx, |ui| {
                let Some(ref mut nes) = self.nes else { return; };
                let mut bus = nes.cpu().bus().borrow_mut();
                let ram = bus.get_mut_cpu_ram();
                const PAGE_SIZE: usize = 256;
                const RAM_SIZE: usize = 2048;
                const PAGE_NUMBER: usize = RAM_SIZE / PAGE_SIZE;
                let current_page = self.debug_state.ram_page_number;
                
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("-1").clicked() {
                            self.debug_state.ram_page_number = current_page.saturating_sub(1);
                        }
                        
                        ui.label(format!("Page #{}", current_page));
                        
                        if ui.button("+1").clicked() {
                            self.debug_state.ram_page_number = std::cmp::min(PAGE_NUMBER-1, current_page + 1);
                        }
                    })
                });

                ui.checkbox(&mut self.debug_state.ram_editing, "edit RAM");

                egui::Grid::new("ram_state_grid")
                    .num_columns(17)
                    .min_col_width(0.0)
                    .spacing([0.0, 0.0])
                    .show(ui, |ui| {
                        ui.label("");
                        for i in 0..16 {
                            ui.label(egui::RichText::new(format!(" {:02X}", i)).monospace().strong());
                        }
                        ui.end_row();

                        for i in 0..16 {
                            ui.label(egui::RichText::new(
                                format!("${:04X}", current_page * PAGE_SIZE + i * 16)
                            ).monospace().strong());

                            for j in 0..16 {
                                let cell_index = current_page * PAGE_SIZE + i * 16 + j;
                                
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
}


// * utilities
fn monospace_label(ui: &mut Ui, text: String) {
    ui.label(egui::RichText::new(text).font(FontId::monospace(16.0)));
}