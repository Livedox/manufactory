use std::sync::{Arc, Mutex};
use graphics_engine::texture::TextureAtlas;
use crate::{player::inventory::PlayerInventory};

pub trait Draw {
    fn draw(&mut self, ui: &mut egui::Ui, atals: Arc<TextureAtlas>, inventory: Arc<Mutex<PlayerInventory>>);
}