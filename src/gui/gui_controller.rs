use std::{borrow::BorrowMut, sync::Arc, time::SystemTime};

use egui::{Align2, vec2, Context, Align, Color32, epaint::Shadow, Rounding, Margin, RichText, Style, Visuals, style::WidgetVisuals, Widget, Stroke};
use winit::{window::Window, dpi::PhysicalPosition, event_loop::{ControlFlow, EventLoopWindowTarget}};

use crate::{player::player::Player, recipes::{storage::Storage, recipes::RECIPES}, engine::texture::TextureAtlas, setting::Setting, save_load::SettingSave, world::loader::{WorldData, WorldLoader}, level::Level};
use super::{my_widgets::{inventory_slot::inventory_slot, category_change_button::category_change_button, container::container, recipe::recipe, hotbar_slot::hotbar_slot, active_recipe::active_recipe}, theme::DEFAULT_THEME, main_screen::{self, MainScreen, in_game_menu::draw_in_game_menu}, setting::draw_setting};
use chrono::{Utc, TimeZone};
enum Task {
    Hotbar(usize),
    Inventory(usize),
    Storage(usize),
}

pub struct GuiController {
    window: Arc<Window>,
    items_atlas: Arc<TextureAtlas>,
    is_ui: bool,
    pub is_menu: bool,
    is_cursor: bool,
    world_name: String,
    seed: String,
    main_screen: MainScreen,
    is_setting: bool,
}


impl GuiController {
    pub fn new(window: Arc<Window>, items_atlas: Arc<TextureAtlas>) -> Self {
        Self {
            window,
            items_atlas,
            is_ui: true,
            is_menu: false,
            is_setting: false,
            is_cursor: true,
            world_name: String::new(),
            seed: String::new(),
            main_screen: MainScreen::new(),
        }
    }
    pub fn is_ui(&self) -> bool {
        self.is_ui
    }
    pub fn toggle_ui(&mut self) {
        self.is_ui = !self.is_ui;
    }
    pub fn toggle_menu(&mut self) {
        self.is_menu = !self.is_menu;
    }

    pub fn update_cursor_lock(&mut self) {
        if !self.is_cursor {
            let size = self.window.inner_size();
            let position = PhysicalPosition::new(size.width as f32/2.0, size.height as f32/2.0);
            self.window.set_cursor_position(position).unwrap();
        };
    }

    pub fn set_cursor_lock(&mut self, is_cursor: bool) {
        self.is_cursor = is_cursor;
        use winit::window::CursorGrabMode;
        let mode = if is_cursor {CursorGrabMode::None} else {CursorGrabMode::Confined};
        
        self.window.set_cursor_grab(mode).unwrap();
        self.window.set_cursor_visible(is_cursor);
    }

    pub fn is_cursor(&self) -> bool { self.is_cursor }

    pub fn draw_main_screen(
        &mut self,
        ctx: &Context,
        window_target: &EventLoopWindowTarget<()>,
        world_loader: &mut WorldLoader,
        setting: &mut Setting,
        level: &mut Option<Level>
    ) -> &mut Self {
        self.main_screen.draw(ctx, window_target, world_loader, setting, level, &mut self.is_setting);
        self
    }

    pub fn draw_setting(&mut self, ctx: &Context, setting: &mut Setting, save: &SettingSave) -> &mut Self {
        draw_setting(ctx, &mut self.is_setting, setting, save);
        self
    }

    pub fn draw_in_game_menu(&mut self, ctx: &Context, exit_level: &mut bool) -> &mut Self {
        draw_in_game_menu(ctx, exit_level, &mut self.is_setting, &mut self.is_menu);
        self
    }

    pub fn draw_inventory(&mut self, ctx: &Context, player: &mut Player) -> &mut Self {
        if !self.is_ui || self.is_menu {return self}
        let mut task: Option<Task> = None;
        let inventory = player.inventory();
        let storage = player.open_storage.as_mut().and_then(|op| op.upgrade());
        egui::Area::new("hotbar_area")
            .anchor(Align2::CENTER_BOTTOM, vec2(1.0, -1.0))
            .show(ctx, |ui| {
                ui.set_visible(self.is_ui);
                let storage = player.open_storage.as_mut().and_then(|op| op.upgrade());
                
                ui.horizontal_top(|ui| {
                    for (i, item) in player.inventory().lock().unwrap().storage().iter().take(10).enumerate() {
                        if ui.add(hotbar_slot(&self.items_atlas, item, player.active_slot == i)).drag_started() {
                            if storage.is_some() {
                                task = Some(Task::Storage(i));
                            } else {
                                task = Some(Task::Inventory(i));
                            }
                        }
                    }
                });
            });
        if !player.is_inventory {return self};
        egui::Area::new("inventory_area")
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_visible(self.is_ui & player.is_inventory);
                if let Some(storage) = &player.open_storage {
                    if let Some(up) = storage.upgrade() {
                        up.lock().unwrap().draw(ui, self.items_atlas.clone(), inventory.clone());
                    }
                }
                let inventory_len = inventory.clone().lock().unwrap().storage().len();
                ui.horizontal(|ui| {        
                    ui.vertical(|ui| {
                        ui.add_space(60.0);
                        for i in 1..=(inventory_len / 10) {
                            ui.horizontal(|ui| {
                                for j in 0..std::cmp::min(inventory_len-10*i, 10) {
                                    if ui.add(inventory_slot(&self.items_atlas, &inventory.clone().lock().unwrap().storage()[i*10 + j])).clicked() {
                                        if storage.is_some() {
                                            task = Some(Task::Storage(i*10 + j));
                                        } else {
                                            task = Some(Task::Hotbar(i*10 + j));
                                        }
                                    };
                                }
                            });
                        }
                    });
                    egui::Frame::none()
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(category_change_button());
                                    ui.add(category_change_button());
                                });
                                
                                ui.add(container(|ui| {
                                    let style = egui::Style {
                                        spacing: egui::style::Spacing { item_spacing: vec2(8.0, 8.0), ..Default::default() },
                                        ..Default::default()
                                    };
                                    ui.set_style(style);
                                    ui.vertical(|ui| {
                                        for i in 0..=(RECIPES().all.len()/5) {
                                            ui.horizontal(|ui| {
                                                for i in RECIPES().all.iter().skip(5*i).take(5) {
                                                    if ui.add(recipe(&self.items_atlas, i)).drag_started() {
                                                        player.inventory().lock().unwrap().start_recipe(i);
                                                    };
                                                }
                                            });
                                        }
                                    });
                                }, Some([280.0, 300.0])));
                            });
                        });
                });   
            });
        if let Some(task) = task {
            match task {
                Task::Hotbar(i) => {inventory.lock().unwrap().place_in_hotbar(i);},
                Task::Inventory(i) => {inventory.lock().unwrap().place_in_inventory(i);},
                Task::Storage(i) => {
                    let Some(item) = inventory.lock().unwrap().mut_storage()[i].0.take() else {return self};
                    let remainder = storage.unwrap().lock().unwrap().add(&item, true);
                    if let Some(r) = remainder {inventory.lock().unwrap().set(&r, i)}
                },
            }
        }
        self
    }


    pub fn draw_debug(&mut self, ctx: &Context, debug_data: &str, debug_block_id: &mut Option<u32>) -> &mut Self {
        if !self.is_ui {return self}
        egui::Window::new("Debug")
            .anchor(Align2([Align::RIGHT, Align::TOP]), vec2(0.0, 20.0))
            .resizable(false)
            .default_width(300.0)
            .frame(
                egui::Frame::none()
                    .fill(DEFAULT_THEME.background)
                    .shadow(Shadow {
                        extrusion: 8.0,
                        color: Color32::from_black_alpha(125),
                    })
                    .rounding(Rounding::same(5.0))
                    .inner_margin(Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                ui.colored_label(DEFAULT_THEME.on_background, debug_data);
                let button = egui::Button::new(
                    RichText::new(format!("{}", debug_block_id.map_or(-1, |a| a as i32)))
                        .color(DEFAULT_THEME.on_primary))
                        .fill(DEFAULT_THEME.primary);
                if ui.add(button).clicked() {
                    if let Some(block_id) = debug_block_id {
                        *block_id += 1;
                        if *block_id > 22 {
                            *debug_block_id = None;
                        }
                    } else {
                        *debug_block_id = Some(0);
                    }
                }
            });
        self
    }


    pub fn draw_active_recieps(&mut self, ctx: &Context, player: &mut Player) -> &mut Self {
        let binding = player.borrow_mut().inventory();
        let mut inventory = binding.lock().unwrap();
        let active = inventory.active_recipe();
        let mut cancel_index: Option<usize> = None;
        egui::Area::new("active_recieps_area")
            .anchor(Align2::LEFT_BOTTOM, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_visible(self.is_ui);
                ui.horizontal(|ui| {
                    active.iter().enumerate().for_each(|(i, recipe)| {
                        ui.add_space(5.0);
                        if ui.add(active_recipe(&self.items_atlas, recipe)).drag_started() {
                            cancel_index = Some(i);
                        };
                    });
                })
            });
        if let Some(index) = cancel_index {
            inventory.cancel_active_recipe(index);
        }
        self
    }
}