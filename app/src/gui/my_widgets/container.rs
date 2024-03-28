use egui::{Rect, Ui};

use crate::{gui::theme::DEFAULT_THEME};


fn container_ui(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut Ui), size: Option<[f32; 2]>) -> egui::Response {
    let width = if let Some(size) = size { size[0] } else { 0.0 };
    let height = if let Some(size) = size { size[1] } else { 0.0 };

    let cursor_rect = ui.cursor();
    let recipe_rect = Rect {
        min: egui::Pos2 { x: cursor_rect.left(), y: cursor_rect.top() },
        max: egui::Pos2 { x: cursor_rect.left()+width, y: cursor_rect.top()+height }
    };
    // Paint rectangle
    ui.painter().rect_filled(recipe_rect, 0.0, DEFAULT_THEME.background);
    egui::Frame::none()
        .inner_margin(5.0)
        .fill(DEFAULT_THEME.background)
        .show(ui, add_contents);
    
    let final_rect = ui.cursor();
    let mut left = width - final_rect.left() + cursor_rect.left();
    let mut top = height - final_rect.top() + cursor_rect.top();
    if left < 0.0 {left = 0.0};
    if top  < 0.0 {top  = 0.0};

    let desired_size = egui::vec2(left, top);
    let (_, response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());
    response
}


pub fn container(add_contents: impl FnOnce(&mut Ui), size: Option<[f32; 2]>) -> impl egui::Widget {
    move |ui: &mut egui::Ui| container_ui(ui, add_contents, size)
}