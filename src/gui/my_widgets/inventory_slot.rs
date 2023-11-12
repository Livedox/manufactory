use egui::{Rect, RichText, vec2, Stroke, pos2};

use crate::{gui::theme::DEFAULT_THEME, texture::TextureAtlas, recipes::item::{Item, PossibleItem}};


const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 50.0;
const FONT_SIZE: f32 = 20.0;
const BOTTOM_PADDING: f32 = 3.0;
const STROKE_WIDTH: f32 = 2.0;


fn inventory_slot_ui(ui: &mut egui::Ui, texture_atlas: &TextureAtlas, item: &PossibleItem) -> egui::Response {
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

    if response.clicked() {
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let left_top = rect.left_top();
        let inventory_slot_rect = Rect::from_min_max(left_top, pos2(left_top.x+WIDTH, left_top.y+HEIGHT));
        // Paint rectangle
        let theme = if is_hover {DEFAULT_THEME.on_background} else {DEFAULT_THEME.background};
        let stroke = Stroke {color: DEFAULT_THEME.on_background, width: STROKE_WIDTH};
        ui.painter().rect(inventory_slot_rect, 0.0, theme, stroke);

        if let Some(item) = item.0 {
            // Paint image
            let uv_rect = texture_atlas.uv_rect(item.id());
            let image = egui::Image::new(egui::load::SizedTexture::new(texture_atlas.texture_id, vec2(WIDTH, HEIGHT)))
                .uv(Rect::from_min_max(pos2(uv_rect.0, uv_rect.1), pos2(uv_rect.2, uv_rect.3)));
            ui.put(inventory_slot_rect, image);

            if item.count > 1 {
                // Paint number
                let count_text = RichText::new(format!("{}", item.count)).size(20.0).color(DEFAULT_THEME.on_background).strong();
                let label = egui::Label::new(count_text);
                ui.put(Rect {
                    min: egui::Pos2 { x: rect.left(), y: rect.bottom()-FONT_SIZE-BOTTOM_PADDING },
                    max: egui::Pos2 { x: rect.left()+WIDTH, y: rect.bottom() }
                }, label);
            }
        } 
    }
    response
}


pub fn inventory_slot<'a>(texture_atlas: &'a TextureAtlas , item: &'a PossibleItem) -> impl egui::Widget+ 'a {
    move |ui: &mut egui::Ui| inventory_slot_ui(ui, texture_atlas, item)
}