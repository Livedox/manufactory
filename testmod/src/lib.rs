use std::sync::{Arc, Mutex, Weak};

use app::{direction::Direction, gui::draw::Draw, player_unlockable, voxels::live_voxels::{DesiarializeLiveVoxel, LiveVoxelBehavior, NewLiveVoxel, PlayerUnlockable}, Registrator};

#[no_mangle]
pub fn super_ultra_test() -> app::coords::global_coord::GlobalCoord {
    println!("Work!");
    app::coords::global_coord::GlobalCoord::new(1, 2, 3)
}

#[no_mangle]
pub fn init(registrator: &mut Registrator) {
    registrator.c.insert(String::from("trashcan"), &create);
    registrator.from_bytes.insert(String::from("trashcan"), &from_bytes);
}

#[no_mangle]
fn create(direction: &Direction) -> Box<dyn LiveVoxelBehavior> {
    Box::new(A(Arc::new(Mutex::new(TrashCan {}))))
}

#[no_mangle]
fn from_bytes(bytes: &[u8]) -> Box<dyn LiveVoxelBehavior> {
    Box::new(A(Arc::new(Mutex::new(TrashCan {}))))
}

#[derive(Debug)]
pub struct TrashCan {}

impl Draw for TrashCan {
    fn draw(&mut self, ui: &mut egui::ui::Ui, _: Arc<app::graphics_engine::texture::TextureAtlas>, _: Arc<Mutex<app::player::inventory::PlayerInventory>>) {
        ui.label("delete");
    }
}

impl PlayerUnlockable for TrashCan {}


#[derive(Debug, Clone)]
pub struct A(Arc<Mutex<TrashCan>>);

impl LiveVoxelBehavior for A {
    fn player_unlockable(&self) -> Option<Weak<Mutex<dyn PlayerUnlockable>>> {
        let tmp: Arc<Mutex<dyn PlayerUnlockable>> = self.0.clone();
        Some(Arc::downgrade(&tmp))
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}