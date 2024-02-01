use std::{collections::{hash_map::DefaultHasher, HashMap}, hash::{Hash, Hasher}};

use chrono::{Utc, TimeZone};
use egui::{vec2, Color32, RichText, Stroke, Ui};

use crate::{world::loader::{WorldData, WorldLoader}, gui::theme::DEFAULT_THEME, level::Level, setting::Setting};

#[derive(Debug, Clone)]
pub struct WorldCreator {
    pub world_name: String,
    pub seed: String,
}

impl WorldCreator {
    pub fn new() -> Self {Self::default()}

    pub fn draw(&mut self, ui: &mut Ui, level: &mut Option<Level>, world_loader: &mut WorldLoader) {
        ui.horizontal(|ui| {
            egui::Frame::none()
                .fill(Color32::WHITE)
                .inner_margin(vec2(3.0, 3.0))
                .rounding(3.0)
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
                        ui.label(RichText::new("World name: ").size(21.0));
                        let world_edit = egui::TextEdit::singleline(&mut self.world_name)
                            .text_color(Color32::BLACK).frame(false);
                        ui.add_sized(vec2(121.0, 20.0), world_edit);
                        ui.label(RichText::new("Seed: ").size(21.0));
                        let seed_edit = egui::TextEdit::singleline(&mut self.seed)
                            .text_color(Color32::BLACK).frame(false);
                        ui.add_sized(vec2(121.0, 20.0), seed_edit);
                    });
                });
            ui.add_space(5.0);
            if ui.button(RichText::new("Create").size(24.0)).clicked() {
                if self.world_name.len() > 0 && self.world_name.len() < 50 {
                    let seed = if self.seed.len() > 0 {
                        let mut hasher = DefaultHasher::new();
                        self.seed.hash(&mut hasher);
                        let hash = hasher.finish();
                        self.seed.parse::<u64>().unwrap_or(hash)
                    } else {
                        rand::random::<u64>()
                    };

                    let _ = world_loader.create_world(&self.world_name, seed);
                    self.world_name = String::new();
                    self.seed = String::new();
                }
            };
        });
    }
}

impl Default for WorldCreator {
    fn default() -> Self {
        Self {
            world_name: String::new(),
            seed: String::new(),
        }
    }
}


pub(crate) fn draw_world_display(ui: &mut Ui, world: &WorldData, level: &mut Option<Level>, setting: &Setting, remove_world: &mut Option<String>, block_texture_id: &HashMap<String, u32>) {
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
            *level = Some(Level::new(&world.name, &setting, block_texture_id));
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
            *remove_world = Some(world.name.clone());
        }
    });
}