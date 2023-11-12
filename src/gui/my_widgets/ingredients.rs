use egui::{Rect, pos2, vec2, Stroke, RichText, TextStyle, Color32, Style, Context, Align2, epaint::Shadow, Rounding, Margin, Id};

use crate::{texture::TextureAtlas, recipes::recipe::Recipe, gui::theme::DEFAULT_THEME};

use crate::gui::my_widgets::container::container;

const WIDTH: f32 = 50.0;
const HEIGHT: f32 = 50.0;
const FONT_SIZE: f32 = 25.0;
const BOTTOM_PADDING: f32 = 3.0;


pub fn ingredients(ui: &mut egui::Ui, texture_atlas: &TextureAtlas, recipe: &Recipe) {
    egui::Area::new("Recipe")
        .fixed_pos(ui.next_widget_position())
        .order(egui::Order::Tooltip)
        .show(ui.ctx(), |ui| {ui.add(container(|ui| {
            let mut rect = ui.cursor();
            rect.max = pos2(
                rect.min.x + WIDTH,
                rect.min.y + HEIGHT,
            );

            let left_top = rect.left_top();
            for (i, item) in recipe.ingredients.iter().enumerate() {
                let inventory_slot_rect = Rect::from_min_max(
                    pos2(left_top.x+WIDTH*i as f32, left_top.y),
                    pos2(left_top.x+WIDTH*(i+1) as f32, left_top.y+HEIGHT)
                );

                // Paint image
                let uv_rect = texture_atlas.uv_rect(item.id());
                let image = egui::Image::new(egui::load::SizedTexture::new(texture_atlas.texture_id, vec2(WIDTH, HEIGHT)))
                    .uv(Rect::from_min_max(pos2(uv_rect.0, uv_rect.1), pos2(uv_rect.2, uv_rect.3)));
                ui.put(inventory_slot_rect, image);

                if item.count > 1 {
                    // Paint number
                    let count_text = RichText::new(format!("{}", item.count))
                        .size(FONT_SIZE)
                        .color(Color32::WHITE)
                        .strong();
                    let label = egui::Label::new(count_text);
                    ui.put(Rect {
                        min: egui::Pos2 { x: rect.left() + WIDTH*i as f32, y: rect.bottom()-FONT_SIZE-BOTTOM_PADDING },
                        max: egui::Pos2 { x: rect.left() + WIDTH*(i+1) as f32, y: rect.bottom() }
                    }, label);
                }
            }
        }, None));});
}