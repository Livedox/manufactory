use chrono::{Utc, TimeZone};
use egui::{vec2, Color32, RichText, Ui, Stroke};

use crate::{world::loader::WorldData, gui::theme::DEFAULT_THEME};

pub(crate) fn draw_world_creation(ui: &mut Ui, world_edit: &mut String, seed_edit: &mut String) {
    ui.horizontal(|ui| {
        egui::Frame::none()
            .fill(Color32::WHITE)
            .inner_margin(vec2(3.0, 3.0))
            .rounding(3.0)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
                    ui.label(RichText::new("World name: ").size(21.0));
                    let world_edit = egui::TextEdit::singleline(world_edit)
                        .text_color(Color32::BLACK).frame(false);
                    ui.add_sized(vec2(121.0, 20.0), world_edit);
                    ui.label(RichText::new("Seed: ").size(21.0));
                    let seed_edit = egui::TextEdit::singleline(seed_edit)
                        .text_color(Color32::BLACK).frame(false);
                    ui.add_sized(vec2(121.0, 20.0), seed_edit);
                });
            });
        ui.add_space(5.0);
        if ui.button(RichText::new("Create").size(24.0)).clicked() {
            println!("Create world!");
        };
    });
}


pub(crate) fn draw_world_display(ui: &mut Ui, world: &WorldData) {
    egui::Frame::none()
        .fill(Color32::WHITE)
        .outer_margin(vec2(0.0, 0.0))
        .inner_margin(vec2(3.0, 3.0))
        .rounding(3.0)
        .show(ui, |ui| {
            egui::Resize::default()
                .fixed_size(vec2(300.0, 30.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let name = egui::RichText::new(&world.name)
                                .size(19.0);
                            ui.heading(name);
                            ui.horizontal(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Seed: ");
                                    ui.label(world.seed.to_string());
                                });
                                ui.add_space(ui.available_width());
                                let time = format!("{}", Utc.timestamp_opt(world.creation_time as i64, 0).unwrap()
                                    .format("%Y-%m-%d"));
                                let time = egui::RichText::new(time)
                                    .size(17.0);
                                ui.label(time);
                            });
                        });
                    });
                });
        });
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.horizontal_top(|ui| {
        ui.add_space(5.0);
        let text = egui::RichText::new("▶")
            .color(DEFAULT_THEME.on_green)
            .size(37.0);
        let button = egui::Button::new(text)
            .min_size(vec2(54.0, 54.0))
            .fill(DEFAULT_THEME.green)
            .stroke(Stroke::NONE);
        if ui.add(button).clicked() {
            println!("Run");
        }
        ui.add_space(5.0);
        let text = egui::RichText::new("🗑")
            .color(DEFAULT_THEME.on_red)
            .size(33.0);
        let button = egui::Button::new(text)
            .min_size(vec2(54.0, 54.0))
            .fill(DEFAULT_THEME.red)
            .stroke(Stroke::NONE);
        if ui.add_sized(vec2(54.0, 54.0), button).clicked() {
            println!("Delete");
        }
    });
}