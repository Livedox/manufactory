use std::sync::{Arc, Mutex, Weak};

use app::{direction::Direction, gui::draw::Draw, player_unlockable, voxels::live_voxels::{LiveVoxelBehavior, PlayerUnlockable}};

#[no_mangle]
pub fn super_ultra_test() -> app::coords::global_coord::GlobalCoord {
    println!("Work!");
    app::coords::global_coord::GlobalCoord::new(1, 2, 3)
}

#[no_mangle]
fn create(direction: &Direction) -> Box<dyn LiveVoxelBehavior> {
    Box::new(TrashCan {})
}

#[no_mangle]
fn from_bytes(bytes: &[u8]) -> Box<dyn LiveVoxelBehavior> {
    Box::new(TrashCan {})
}

#[derive(Debug)]
pub struct TrashCan {}

impl Draw for TrashCan {
    fn draw(&mut self, ui: &mut egui::Ui, atals: Arc<TextureAtlas>, inventory: Arc<Mutex<app::player::inventory::PlayerInventory>>) {
        ui.text("delete!");
    }
}

impl PlayerUnlockable for TrashCan {

}


#[derive(Debug)]
pub struct A(Arc<Mutex<TrashCan>>);

impl LiveVoxelBehavior for A {
    player_unlockable!();
    
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}