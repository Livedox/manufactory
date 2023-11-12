use egui::{Rect, vec2, Stroke, TextureId, pos2, load::SizedTexture};

use crate::{gui::theme::DEFAULT_THEME, recipes::recipe::Recipe, texture::TextureAtlas};

use super::ingredients::ingredients;

const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 50.0;
const STROKE_WIDTH: f32 = 2.0;
const PADDING: f32 = 3.0;
const IMAGE_WIDTH: f32 = WIDTH - PADDING*2.0;
const IMAGE_HEIGHT: f32 = HEIGHT - PADDING*2.0;

fn recipe_ui(ui: &mut egui::Ui, texture_atlas: &TextureAtlas, recipe: &Recipe) -> egui::Response {
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
        let left_top = rect.left_top();
        let mut recipe_rect = Rect::from_min_max(left_top, pos2(left_top.x+WIDTH, left_top.y+HEIGHT));
        // Paint rectangle
        let theme = if is_hover {DEFAULT_THEME.on_background} else {DEFAULT_THEME.background};
        let stroke = Stroke {color: DEFAULT_THEME.on_background, width: STROKE_WIDTH};   
        ui.painter().rect(recipe_rect, 6.0, theme, stroke);

        // Paint image
        recipe_rect.min = pos2(recipe_rect.min.x+PADDING, recipe_rect.min.y+PADDING);
        recipe_rect.max = pos2(recipe_rect.max.x-PADDING, recipe_rect.max.y-PADDING);
        let uv_rect = texture_atlas.uv_rect(recipe.result.id());
        let image = egui::Image::new(egui::load::SizedTexture::new(texture_atlas.texture_id, vec2(IMAGE_WIDTH, IMAGE_HEIGHT)))
            .uv(Rect::from_min_max(pos2(uv_rect.0, uv_rect.1), pos2(uv_rect.2, uv_rect.3)));
        ui.put(recipe_rect, image);
    }

    if is_hover {
        ingredients(ui, texture_atlas, recipe);
    }

    response
}


pub fn recipe<'a>(texture_atlas: &'a TextureAtlas, recipe: &'a Recipe) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| recipe_ui(ui, texture_atlas, recipe)
}