

use egui::{vec2, Align2};
use winit::event_loop::{EventLoopWindowTarget};
use crate::{world::loader::{WorldLoader}, setting::Setting, level::Level};
use crate::Indices;
use self::worlds::{draw_world_display, WorldCreator};



pub mod button;
pub mod worlds;
pub mod in_game_menu;

#[derive(Clone, Debug)]
pub struct MainScreen {
    is_worlds: bool,
    world_creator: WorldCreator,
}

impl MainScreen {
    pub fn new() -> Self {Self::default()}

    pub fn draw(
      &mut self, 
      ctx: &egui::Context,
      window_target: &EventLoopWindowTarget<()>,
      worlds: &mut WorldLoader,
      setting: &mut Setting,
      level: &mut Option<Level>,
      is_setting: &mut bool,
      indices: &Indices
    ) {
        self.draw_main_screen(ctx, window_target, is_setting);
        self.draw_worlds(ctx, worlds, level, setting, indices);
    }

    fn draw_main_screen(&mut self, ctx: &egui::Context, window_target: &EventLoopWindowTarget<()>, is_setting: &mut bool) {
        if self.is_worlds || *is_setting {return};
        egui::Area::new("MainScreen")
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.visuals_mut().widgets.hovered = ui.visuals().widgets.inactive;
                ui.style_mut().spacing.item_spacing = vec2(0.0, 4.0);
                // ADD IN FUTTURE
                // if ui.add(self::button::continue_button()).clicked() {
                //     println!("Clicked");
                // };
                if ui.add(self::button::button("Play")).clicked() {
                    self.is_worlds = true;
                };
                if ui.add(self::button::button("Setting")).clicked() {
                    *is_setting = !*is_setting;
                };
                if ui.add(self::button::exit()).clicked() {
                    window_target.exit();
                };
            });
    }


    fn draw_worlds(&mut self, ctx: &egui::Context, world_loader: &mut WorldLoader, level: &mut Option<Level>, setting: &Setting, indices: &Indices) {
        let mut remove_world = None;
        egui::Window::new("Worlds")
            .open(&mut self.is_worlds)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_TOP, vec2(0.0, 20.0))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 7.0;
                self.world_creator.draw(ui, level, world_loader);
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        world_loader.worlds.iter().for_each(|world| {
                            ui.horizontal_top(|ui| {
                                draw_world_display(ui, world, level, setting, &mut remove_world, indices);
                            });
                        });
                    });
            });
        if let Some(name) = remove_world {
            world_loader.remove_world(&name).unwrap();
        }
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self {
            is_worlds: false,
            world_creator: WorldCreator::new(),
        }
    }
}