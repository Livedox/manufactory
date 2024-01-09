use egui::{Stroke, Color32};

use crate::gui::theme::DEFAULT_THEME;

const WIDTH: f32 = 250.0;
const HEIGHT: f32 = 40.0;
const FONT_SIZE: f32 = 20.0;

pub fn continue_button<'a>() -> egui::Button<'a> {
    let text = egui::RichText::new("Continue")
        .color(DEFAULT_THEME.on_green)
        .size(FONT_SIZE);
    egui::Button::new(text)
        .min_size(egui::vec2(WIDTH, HEIGHT))
        .fill(DEFAULT_THEME.green)
        .stroke(Stroke::new(0.0, Color32::WHITE))
}

pub fn button(text: &str) -> egui::Button {
    let text = egui::RichText::new(text)
        .size(FONT_SIZE);
    egui::Button::new(text)
        .min_size(egui::vec2(WIDTH, HEIGHT))
        .stroke(Stroke::new(0.0, Color32::WHITE))        
}

pub fn exit<'a>() -> egui::Button<'a> {
    let text = egui::RichText::new("Exit")
        .color(DEFAULT_THEME.on_red)
        .size(FONT_SIZE);
    egui::Button::new(text)
        .min_size(egui::vec2(WIDTH, HEIGHT))
        .fill(DEFAULT_THEME.red)
        .stroke(Stroke::new(0.0, Color32::WHITE))      
}