use egui::{Rect, RichText, Stroke, Rounding};

use crate::{gui::theme::DEFAULT_THEME};

const WIDTH: f32 = 120.0;
const HEIGHT: f32 = 60.0;
const FONT_SIZE: f32 = 20.0;
const BOTTOM_PADDING: f32 = 3.0;
const STROKE_WIDTH: f32 = 2.0;


fn category_change_button_ui(ui: &mut egui::Ui) -> egui::Response {
    let mut is_hover = false;
    let desired_size = egui::vec2(WIDTH, HEIGHT);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

    if response.hovered() {
        is_hover = true;
        response.mark_changed();
    }

    if response.drag_started() {
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let category_change_button_rect = Rect {
            min: egui::Pos2 { x: rect.left(), y: rect.top() },
            max: egui::Pos2 { x: rect.left()+WIDTH, y: rect.top()+HEIGHT }
        };
        // Paint rectangle
        let theme = if is_hover {DEFAULT_THEME.on_background} else {DEFAULT_THEME.background};
        let stroke = Stroke {color: DEFAULT_THEME.on_background, width: STROKE_WIDTH};
        let rounding = Rounding {ne: 8.0, nw: 8.0, ..Default::default()};
        ui.painter().rect(category_change_button_rect, rounding, theme, stroke);
        let count_text = RichText::new("ALL").size(20.0).color(DEFAULT_THEME.on_background).strong();
        let label = egui::Label::new(count_text);
        ui.put(Rect {
            min: egui::Pos2 { x: rect.left(), y: rect.top() },
            max: egui::Pos2 { x: rect.left()+WIDTH, y: rect.bottom() }
        }, label);
    }
    response
}


pub fn category_change_button<'a>() -> impl egui::Widget+ 'a {
    category_change_button_ui
}