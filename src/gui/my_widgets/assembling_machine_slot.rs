use egui::{Rect, RichText, vec2, Stroke, pos2, Color32};

use crate::{gui::theme::DEFAULT_THEME, texture::TextureAtlas, recipes::{item::{Item, PossibleItem}, recipe::Recipe}};

const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 50.0;
const FONT_SIZE: f32 = 20.0;
const BOTTOM_PADDING: f32 = 3.0;
const STROKE_WIDTH: f32 = 2.0;

const GREY: Color32 = Color32::from_rgb(50,50,50);


fn assembling_machine_slot_ui(ui: &mut egui::Ui, texture_atlas: &TextureAtlas, item: &PossibleItem, slot_id: usize, recipe: &Recipe, result: bool) -> egui::Response {
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
        let theme = match (is_hover, item.0.is_none()) {
            (true, _) => DEFAULT_THEME.on_background,
            (_, true) => GREY,
            _ => DEFAULT_THEME.background,
        };
        // let theme = if is_hover {DEFAULT_THEME.on_background} else {DEFAULT_THEME.background};
        let stroke = Stroke {color: DEFAULT_THEME.on_background, width: STROKE_WIDTH};
        ui.painter().rect(inventory_slot_rect, 0.0, theme, stroke);

        let is_not_item = item.0.is_none();
        let item_id = item.0
            .map_or_else(
                || if !result {
                    recipe.ingredients
                        .get(slot_id)
                        .and_then(|i| Some(i.id()))
                } else {
                    Some(recipe.result.id())
                },
                |i| Some(i.id()));
        if let Some(item_id) = item_id {
            // Paint image
            let uv_rect = texture_atlas.uv_rect(item_id);
            let mut image = egui::Image::new(egui::load::SizedTexture::new(texture_atlas.texture_id, vec2(WIDTH, HEIGHT)))
                .uv(Rect::from_min_max(pos2(uv_rect.0, uv_rect.1), pos2(uv_rect.2, uv_rect.3)));
            if is_not_item {image = image.tint(GREY)}
            ui.put(inventory_slot_rect, image);

            if let Some(item) = item.0 {
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
    }
    response
}


pub fn assembling_machine_slot<'a>(texture_atlas: &'a TextureAtlas, item: &'a PossibleItem, slot_id: usize, recipe: &'a Recipe, result: bool) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| assembling_machine_slot_ui(ui, texture_atlas, item, slot_id, recipe, result)
}