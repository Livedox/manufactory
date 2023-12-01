use std::sync::{Arc, Mutex};

use crate::{player::inventory::PlayerInventory, engine::texture::TextureAtlas};

pub trait Draw {
    fn draw(&mut self, ui: &mut egui::Ui, atals: Arc<TextureAtlas>, inventory: Arc<Mutex<PlayerInventory>>) {}
}