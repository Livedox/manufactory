use egui::{Context, Align2, vec2};
use crate::gui::main_screen::button;


pub fn draw_in_game_menu(ctx: &Context, exit_level: &mut bool, is_setting: &mut bool, is_menu: &mut bool) {
    if !*is_menu {return};
    egui::Area::new("InGameMenu")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.visuals_mut().widgets.hovered = ui.visuals().widgets.inactive;
            ui.style_mut().spacing.item_spacing = vec2(0.0, 4.0);
            if ui.add(button::continue_button()).clicked() {
                *is_menu = false;
            };
            if ui.add(button::button("Setting")).clicked() {
                *is_setting = !*is_setting;
            };
            if ui.add(button::exit()).clicked() {
                *exit_level = true;
            };
        });
}