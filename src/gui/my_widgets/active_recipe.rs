use egui::{Rect, vec2, Stroke, Pos2, pos2, Color32, epaint};
use crate::{gui::theme::DEFAULT_THEME, recipes::recipe::ActiveRecipe, engine::texture::TextureAtlas};
use std::f32::consts::PI;

const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 50.0;


fn active_recipe_ui(ui: &mut egui::Ui, texture_atlas: &TextureAtlas, active_recipe: &ActiveRecipe) -> egui::Response {
    let desired_size = egui::vec2(WIDTH, HEIGHT);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

    if response.drag_started() { response.mark_changed(); }


    if ui.is_rect_visible(rect) {
        let left_top = rect.left_top();
        let mut recipe_rect = Rect::from_min_max(left_top, pos2(left_top.x+WIDTH, left_top.y+HEIGHT));
        // Paint rectangle     
        ui.painter().rect_filled(recipe_rect, 0.0, DEFAULT_THEME.background);

        // Paint image
        let uv_rect = texture_atlas.uv_rect(active_recipe.recipe.result.id());
        let image = egui::Image::new(egui::load::SizedTexture::new(texture_atlas.texture_id, vec2(WIDTH, HEIGHT)))
            .uv(Rect::from_min_max(pos2(uv_rect.0, uv_rect.1), pos2(uv_rect.2, uv_rect.3)));
        ui.put(recipe_rect, image);

        {
            let angle = 2.0*PI*active_recipe.progress();
            let n = 4;

            let mut points: Vec<Pos2> = Vec::with_capacity(n+2);
            let center = (WIDTH/2.0+rect.left(), HEIGHT/2.0+rect.top());
            points.push(pos2(center.0, center.1));
            for i in 0..=n {
                let current_angle = i as f32*angle/n as f32;
                let x = (current_angle.cos()+0.5)*WIDTH+rect.left();
                let y = (current_angle.sin()+0.5)*HEIGHT+rect.top();
                points.push(pos2(x, y));
            }
            points.push(pos2(center.0, center.1));

            let shape = epaint::PathShape::convex_polygon(points, Color32::from_rgba_unmultiplied(255,255,255,160), Stroke::NONE);
            // + 1.0 to fix background creep 
            recipe_rect.max = pos2(recipe_rect.max.x+1.0, recipe_rect.max.y+1.0);
            ui.painter().with_clip_rect(recipe_rect).add(shape);
        }
    }
    response
}


pub fn active_recipe<'a>(texture_atlas: &'a TextureAtlas, active_recipe: &'a ActiveRecipe) -> impl egui::Widget+ 'a {
    move |ui: &mut egui::Ui| active_recipe_ui(ui, texture_atlas, active_recipe)
}