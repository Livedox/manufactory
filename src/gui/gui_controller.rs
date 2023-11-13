use std::{rc::{Rc, Weak}, cell::RefCell, borrow::BorrowMut};

use egui::{Align2, vec2, Context, Align, Color32, epaint::Shadow, Rounding, Margin, RichText, Ui};
use winit::{window::Window, dpi::PhysicalPosition};

use crate::{player::{inventory::PlayerInventory, player::Player}, recipes::{storage::Storage, recipes::RECIPES}, texture::TextureAtlas, voxels::{voxel_data::{VoxelBox, Furnace, PlayerUnlockableStorage}, assembling_machine::AssemblingMachine}};
use super::{my_widgets::{inventory_slot::inventory_slot, category_change_button::category_change_button, container::container, recipe::recipe, hotbar_slot::hotbar_slot, active_recipe::active_recipe, assembling_machine_slot::assembling_machine_slot}, theme::DEFAULT_THEME};

pub fn add_to_storage(src: *mut dyn Storage, dst: *mut dyn Storage, index: usize) {
    let Some(add_item) = (unsafe {src.as_mut().unwrap().mut_storage()[index].0.take()}) else {
        return;
    };
    let Some(remainder) = (unsafe {dst.as_mut().unwrap().add(&add_item, true)}) else {
        return;
    };
    unsafe {src.as_mut().unwrap().set(&remainder, index)};
}

pub struct GuiController {
    window: Rc<Window>,
    items_atlas: Rc<TextureAtlas>,
    is_ui: bool,
    is_inventory: bool,
    is_menu: bool,
    is_cursor: bool,
}


impl GuiController {
    pub fn new(window: Rc<Window>, items_atlas: Rc<TextureAtlas>) -> Self {
        Self {
            window,
            items_atlas,
            is_ui: true,
            is_inventory: true,
            is_menu: false,
            is_cursor: true,
        }
    }
    pub fn is_ui(&self) -> bool {
        self.is_ui
    }
    pub fn toggle_ui(&mut self) {
        self.is_ui = !self.is_ui;
    }
    pub fn toggle_inventory(&mut self) -> bool {
        self.is_inventory = !self.is_inventory;
        self.is_inventory
    }
    pub fn set_inventory(&mut self, state: bool) {
        self.is_inventory = state;
        self.set_cursor_lock(state);
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


    pub fn draw_box(&self, ui: &mut Ui, storage: &Weak<RefCell<VoxelBox>>, inventory: *mut PlayerInventory) {
        let voxel_box = storage.upgrade().unwrap();
        let voxel_box_ptr = voxel_box.as_ptr();
        ui.horizontal(|ui| {ui.vertical(|ui| {
            let len = voxel_box.borrow().storage().len();
            let count = (len as f32 / 10.0).ceil() as usize;
            for i in 0..count {
                ui.horizontal(|ui| {
                    for j in 0..(std::cmp::min(10, len - i*10)) {
                        if ui.add(inventory_slot(&self.items_atlas, &voxel_box.borrow().storage()[i*10 + j])).clicked() {
                            add_to_storage(voxel_box_ptr, inventory, i*10 + j);
                            // if let Some(add_item) = voxel_box.borrow().storage()[i*10 + j].0 {
                            //     if let Some(item) = inventory.as_ref().borrow_mut().add(&add_item, true) {
                            //         voxel_box.as_ref().borrow_mut().set(&item, i*10 + j)
                            //     }
                            // }
                        };
                    }
                });
            }
        })});
    }

    pub fn draw_furnace(&self, ui: &mut Ui, storage: &Weak<RefCell<Furnace>>, inventory_ptr: *mut PlayerInventory) {
        let furnace = storage.upgrade().unwrap();
        let furnace_ptr = furnace.as_ptr();
        ui.horizontal(|ui| {
            for (index, item) in furnace.borrow().storage().iter().enumerate() {
                if ui.add(inventory_slot(&self.items_atlas, item)).drag_started() {
                    add_to_storage(furnace_ptr, inventory_ptr, index)
                }
            }
        });
    }

    pub fn draw_assembling_machine(&self, ui: &mut Ui, storage: &Weak<RefCell<AssemblingMachine>>, inventory_ptr: *mut PlayerInventory) {
        let assembling_machine = storage.upgrade().unwrap();
        let assembling_machine_ptr = assembling_machine.as_ptr();
        let selected_recipe = assembling_machine.borrow().selected_recipe();
        if let Some(selected_recipe) = selected_recipe {
            ui.horizontal(|ui| {
                for (i, item) in assembling_machine.borrow().storage().iter().enumerate() {
                    if ui.add(assembling_machine_slot(&self.items_atlas, item, i, selected_recipe, i==3)).drag_started() {
                        add_to_storage(assembling_machine_ptr, inventory_ptr, i);
                    };
                }
            });
        }
        ui.vertical(|ui| {
            ui.add(container(|ui| {
                let style = egui::Style {
                    spacing: egui::style::Spacing { item_spacing: vec2(8.0, 8.0), ..Default::default() },
                    ..Default::default()
                };
                ui.set_style(style);
                ui.horizontal(|ui| {
                    for i in RECIPES().assembler.all() {
                        if ui.add(recipe(&self.items_atlas, i)).drag_started() {
                            let result = assembling_machine.as_ref().borrow_mut().select_recipe(i.index);
                            for item in result.0 {
                                let Some(item) = item.0 else {continue};
                                unsafe {inventory_ptr.as_mut().unwrap().add(&item, true)};
                            }
                            for item in result.1 {
                                unsafe {inventory_ptr.as_mut().unwrap().add(&item, true)};
                            }
                        };
                    }
                });
            }, None));
        });
    }


    pub fn draw_inventory(&self, ctx: &Context, player: &mut Player, slot_id: usize) -> &Self {
        let inventory = player.inventory();
        let inventory_ptr = inventory.as_ptr();
        egui::Area::new("hotbar_area")
            .anchor(Align2::CENTER_BOTTOM, vec2(1.0, -1.0))
            .show(ctx, |ui| {
                ui.set_visible(self.is_ui);
                let storage = player.open_storage.as_mut().map(|op| op.to_storage().upgrade().unwrap());
                ui.horizontal_top(|ui| {
                    for (i, item) in player.inventory().borrow().storage().iter().take(10).enumerate() {
                        if ui.add(hotbar_slot(&self.items_atlas, item, player.active_slot == i)).drag_started() {
                            if let Some(storage) = &storage {
                                add_to_storage(inventory_ptr, storage.as_ptr(), i);
                            } else {
                                unsafe {inventory_ptr.as_mut().unwrap().place_in_inventory(i);}
                            }
                        }
                    }
                });
            });
        if !self.is_inventory {return self};
        egui::Area::new("inventory_area")
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_visible(self.is_ui & self.is_inventory);
                if let Some(storage) = &player.open_storage {
                    match storage {
                        PlayerUnlockableStorage::VoxelBox(a) => self.draw_box(ui, a, inventory_ptr),
                        PlayerUnlockableStorage::Furnace(a) => self.draw_furnace(ui, a, inventory_ptr),
                        PlayerUnlockableStorage::AssemblingMachine(a) => self.draw_assembling_machine(ui, a, inventory_ptr),
                    }
                }
                let storage = player.open_storage.as_mut().map(|op| op.to_storage().upgrade().unwrap());
                let inventory_len = inventory.borrow().storage().len();
                ui.horizontal(|ui| {        
                    ui.vertical(|ui| {
                        ui.add_space(60.0);
                        for i in 1..=(inventory_len / 10) {
                            ui.horizontal(|ui| {
                                for j in 0..std::cmp::min(inventory_len-10*i, 10) {
                                    if ui.add(inventory_slot(&self.items_atlas, &inventory.borrow().storage()[i*10 + j])).clicked() {
                                        if let Some(storage) = &storage {
                                            add_to_storage(inventory_ptr, storage.as_ptr(), i*10 + j);
                                        } else {
                                            unsafe {inventory_ptr.as_mut().unwrap().place_in_hotbar(i*10 + j);}
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
                                                        player.inventory().as_ref().borrow_mut().start_recipe(i);
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

        self
    }


    pub fn draw_debug(&self, ctx: &Context, debug_data: &str, choosen_block_id: &mut u32) -> &Self {
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
                let button = egui::Button::new(RichText::new(format!("{}", choosen_block_id)).color(DEFAULT_THEME.on_primary)).fill(DEFAULT_THEME.primary);
                if ui.add(button).clicked() {
                    *choosen_block_id += 1;
                    if *choosen_block_id > 17 {
                        *choosen_block_id = 1;
                    }
                }
            });
        self
    }


    pub fn draw_active_recieps(&self, ctx: &Context, player: &mut Player) -> &Self {
        let binding = player.borrow_mut().inventory();
        let mut inventory = binding.as_ref().borrow_mut();
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