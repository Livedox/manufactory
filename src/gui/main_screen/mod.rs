use egui::{vec2, Align2};
use winit::event_loop::ControlFlow;

use crate::{world::loader::WorldData, save_load::SettingSave, setting::Setting};

use self::worlds::{draw_world_display, draw_world_creation};

use super::setting::draw_setting;

pub mod button;
pub mod worlds;

#[derive(Clone, Debug)]
pub struct MainScreen {
    is_setting: bool,
    is_worlds: bool,
    world_edit: String,
    seed_edit: String,
}

impl MainScreen {
    pub fn new() -> Self {Self::default()}

    pub fn draw(
      &mut self, 
      ctx: &egui::Context,
      control_flow: &mut ControlFlow,
      worlds: &[WorldData],
      setting: &mut Setting,
      save: &SettingSave
    ) {
        self.draw_main_screen(ctx, control_flow);
        self.draw_worlds(ctx, worlds);
        draw_setting(ctx, &mut self.is_setting, setting, save);
    }

    fn draw_main_screen(&mut self, ctx: &egui::Context, control_flow: &mut ControlFlow) {
        if self.is_worlds || self.is_setting {return};
        egui::Area::new("MainScreen")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.visuals_mut().widgets.hovered = ui.visuals().widgets.inactive;
                ui.style_mut().spacing.item_spacing = vec2(0.0, 4.0);
                if ui.add(self::button::continue_button()).clicked() {
                    println!("Clicked");
                };
                if ui.add(self::button::button("Play")).clicked() {
                    self.is_worlds = true;
                };
                if ui.add(self::button::button("Setting")).clicked() {
                    self.is_setting = !self.is_setting;
                };
                if ui.add(self::button::exit()).clicked() {
                    *control_flow = ControlFlow::Exit;
                };
            });
    }


    fn draw_worlds(&mut self, ctx: &egui::Context, worlds: &[WorldData]) {
        egui::Window::new("Worlds")
            .open(&mut self.is_worlds)
            .movable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_TOP, vec2(0.0, 20.0))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 7.0;
                draw_world_creation(ui, &mut self.world_edit, &mut self.seed_edit);
                worlds.iter().for_each(|world| {
                    ui.horizontal_top(|ui| {
                        draw_world_display(ui, world);
                    });
                });
            });
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self {
            is_setting: false,
            is_worlds: false,
            world_edit: String::new(),
            seed_edit: String::new()
        }
    }
}