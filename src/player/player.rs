use std::{sync::{Mutex, Arc, Weak}};
use crate::{bytes::{AsFromBytes, BytesCoder}, camera::camera_controller::CameraController, content::Content, coords::global_coord::GlobalCoord, direction::Direction, input_event::{input_service::{InputService, Key}, KeypressState}, recipes::{item_interaction::ItemInteraction, items::ITEMS, storage::Storage}, voxels::live_voxels::PlayerUnlockable, world::World, CAMERA_FAR, CAMERA_FOV, CAMERA_NEAR};
use super::inventory::PlayerInventory;

use nalgebra_glm as glm;

#[derive(Debug)]
pub struct Player {
    position: glm::Vec3,
    camera: CameraController,
    pub is_inventory: bool,
    pub active_slot: usize,
    pub open_storage: Option<Weak<Mutex<dyn PlayerUnlockable>>>,
    inventory: Arc<Mutex<PlayerInventory>>,
}


impl Player {
    const SPEED: f32 = 14.0; //14.0 default

    pub fn new(camera: CameraController, position: glm::Vec3) -> Self {
        Self {
            open_storage: None,
            inventory: Arc::new(Mutex::new(PlayerInventory::new())),
            active_slot: 0,
            position,
            camera,
            is_inventory: true,
        }
    }


    pub fn inventory(&mut self) -> Arc<Mutex<PlayerInventory>> {
        self.inventory.clone()
    }


    pub fn on_right_click(&mut self, world: &World, xyz: &GlobalCoord, dir: &Direction, content: &Content) {
        let Some(item_id) = self.inventory
            .lock().unwrap()
            .storage()[self.active_slot].0
            .map(|item| item.id()) else {return};
        ITEMS()[item_id as usize].on_right_click(world, self, xyz, dir, content);
    }

    pub fn set_open_storage(&mut self, storage: Weak<Mutex<dyn PlayerUnlockable>>) {
        self.open_storage = Some(storage);
        self.is_inventory = true;
    }

    pub fn handle_input(&mut self, input: &InputService, delta_time: f32, is_cursor: bool) {
        if !self.is_inventory && !is_cursor {self.camera.update_rotation(input.delta().0, input.delta().1, delta_time)}

        if input.is_key(&Key::KeyE, KeypressState::AnyJustPress) {
            self.is_inventory = !self.is_inventory;
            if !self.is_inventory {self.open_storage = None};
        }

        if input.is_key(&Key::KeyW, KeypressState::AnyStayPress) {
            self.position +=  self.camera.front() * Self::SPEED * delta_time;
        }
        if input.is_key(&Key::KeyS, KeypressState::AnyStayPress) {
            self.position -=  self.camera.front() * Self::SPEED * delta_time;
        }
        if input.is_key(&Key::KeyA, KeypressState::AnyStayPress) {
            self.position -=  self.camera.right() * Self::SPEED * delta_time;
        }
        if input.is_key(&Key::KeyD, KeypressState::AnyStayPress) {
            self.position +=  self.camera.right() * Self::SPEED * delta_time;
        }
        self.camera.set_position(self.position);

        if input.wheel() < 0 {
            self.active_slot += 1;
            if self.active_slot > 9 {self.active_slot = 0}
        }
        if input.wheel() > 0 {
            if self.active_slot == 0 {
                self.active_slot = 9
            } else {
                self.active_slot -= (input.wheel() > 0) as usize;
            }
        }
        
        
        [Key::Digit1, Key::Digit2, Key::Digit3, Key::Digit4, Key::Digit5,
            Key::Digit6, Key::Digit7, Key::Digit8, Key::Digit9, Key::Digit0]
            .iter().enumerate().for_each(|(i, key)| {
                if input.is_key(key, KeypressState::AnyPress) {
                    self.active_slot = i;
                }
            });
    }

    pub fn camera(&self) -> &CameraController {&self.camera}
    pub fn position(&self) -> &glm::Vec3 {&self.position}
}


unsafe impl Send for Player {}


const PLAYER_FROMAT_VERSION: u32 = 1;
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    format_version: u32,
    slot: u32,
    x: f32,
    y: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
}
impl AsFromBytes for Header {}

impl BytesCoder for Player {
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();
        bytes.extend(Header {
            format_version: PLAYER_FROMAT_VERSION,
            slot: self.active_slot as u32,
            pitch: self.camera.pitch(),
            yaw: self.camera.yaw(),
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
        }.as_bytes());
        bytes.extend(self.inventory.lock().unwrap().encode_bytes().as_ref());
        bytes.into()
    }

    fn decode_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes[0..Header::size()]);
        let inventory = PlayerInventory::decode_bytes(&bytes[Header::size()..]);
        let position = glm::vec3(header.x, header.y, header.z);
        let mut camera = CameraController::new(position, CAMERA_FOV, CAMERA_NEAR, CAMERA_FAR);
        camera.set_angle(header.yaw, header.pitch);
        camera.update_rotation(0.0, 0.0, 0.0);
        Self {
            position,
            camera,
            is_inventory: true,
            active_slot: header.slot as usize,
            open_storage: None,
            inventory: Arc::new(Mutex::new(inventory)),
        }
    }
}