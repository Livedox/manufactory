use std::sync::{Arc, Mutex, Condvar, mpsc::Sender};
use crate::{unsafe_mutex::UnsafeMutex, world::{World, sun::{Sun, Color}, global_coords::GlobalCoords, chunk_coords::ChunkCoords, local_coords::LocalCoords}, threads::{Threads, save::SaveState}, player::player::Player, save_load::WorldSaver, camera, recipes::{storage::Storage, item::Item}, CAMERA_FOV, CAMERA_NEAR, CAMERA_FAR, setting::Setting, voxels::{chunks::WORLD_HEIGHT, ray_cast::ray_cast, block::blocks::BLOCKS}, graphic::{render::RenderResult, render_selection::render_selection}, nalgebra_converter::Conventer, input_event::{input_service::{InputService, Mouse}, KeypressState}, my_time::Time, direction::Direction, engine::state::State, gui::gui_controller::{self, GuiController}};

use nalgebra_glm as glm;

pub struct Level {
    sun: Sun<9>,
    world_saver: WorldSaver,
    pub world: Arc<UnsafeMutex<World>>,
    pub player: Player,
    threads: Option<Threads>,
}

impl Level {
    pub fn new(world_name: &str, setting: &Setting, render_sender: Sender<RenderResult>) -> Self {
        let world_saver = WorldSaver::new(world_name.into());
        let player = match world_saver.player.lock().unwrap().load_player() {
            Some(player) => player,
            _ => {
                let camera = camera::camera_controller::CameraController::new(
                    glm::vec3(0.0, 20.0, 0.0), CAMERA_FOV, CAMERA_NEAR, CAMERA_FAR);
                let mut player = Player::new(camera, glm::vec3(0.0, 20.0, 0.0));
                let binding = player.inventory();
                let mut inventory = binding.lock().unwrap();
                _ = inventory.add_by_index(&Item::new(0, 100), 10);
                _ = inventory.add_by_index(&Item::new(1, 100), 11);
                _ = inventory.add_by_index(&Item::new(2, 100), 12);
                _ = inventory.add_by_index(&Item::new(3, 100), 13);
                player
            }
        };

        let render_diameter = (setting.render_radius * 2 + 1) as i32;
        let chunk_position: ChunkCoords = GlobalCoords::from(player.position().tuple()).into();
        let ox = chunk_position.0 - setting.render_radius as i32;
        let oz = chunk_position.2 - setting.render_radius as i32;
        let world = Arc::new(UnsafeMutex::new(
            World::new(render_diameter, WORLD_HEIGHT as i32, render_diameter, ox, 0, oz)));
        let save_condvar = Arc::new((Mutex::new(SaveState::Unsaved), Condvar::new()));
        
        let threads = Some(Threads::new(
            world.clone(),
            world_saver.regions.clone(),
            render_sender,
            save_condvar.clone()
        ));

        let sun = Sun::new(
            60, [0, 50, 60, 230, 240, 290, 300, 490, 500],
            [Color(1.0, 0.301, 0.0), Color(1.0, 0.654, 0.0),
             Color(1.0, 1.0, 1.0), Color(1.0, 1.0, 1.0),
             Color(1.0, 0.654, 0.0), Color(1.0, 0.301, 0.0),
             Color(0.0, 0.0, 0.0), Color(0.0, 0.0, 0.0),
             Color(1.0, 0.301, 0.0)],
             
            [Color(1.0, 0.301, 0.0), Color(1.0, 0.654, 0.0),
             Color(0.0, 0.513, 0.639), Color(0.0, 0.513, 0.639),
             Color(1.0, 0.654, 0.0), Color(1.0, 0.301, 0.0),
             Color(0.0, 0.0, 0.0), Color(0.0, 0.0, 0.0),
             Color(1.0, 0.301, 0.0)]);

        Self {
            world_saver,
            player,
            sun,
            threads,
            world
        }
    }

    pub fn update(&mut self, input: &InputService, time: &Time, is_cursor: bool, state: &mut State, gui_controller: &mut GuiController, debug_block_id: &mut Option<u32>) {
        self.player.handle_input(input, time.delta(), is_cursor);
        self.player.inventory().lock().unwrap().update_recipe();

        let mut world = unsafe {self.world.lock_unsafe()}.unwrap();
        let result = ray_cast(&world.chunks, &self.player.position().array(),
            &self.player.camera().front_array(), 10.0);

        if let Some(result) = result {
            let (global, voxel, norm) = (result.0, result.1, result.2);
            let global: GlobalCoords = global.into();
            let voxel_id = voxel.map_or(0, |v| v.id) as usize;

            if voxel_id != 0 {
                let min = *BLOCKS()[voxel_id].min_point() + global.into();
                let max = *BLOCKS()[voxel_id].max_point() + global.into();
                state.selection_vertex_buffer =
                    Some(render_selection(
                        state.device(),
                        &min.into(),
                        &max.into()
                    ));
            } else {
                state.selection_vertex_buffer = None;
            }

            if input.is_mouse(&Mouse::Left, KeypressState::AnyJustPress) && !is_cursor {
                BLOCKS()[voxel_id as usize].on_block_break(&mut world, &mut self.player, &global);
            } else if input.is_mouse(&Mouse::Right, KeypressState::AnyJustPress) && !is_cursor {
                let gxyz = global + norm.tuple().into();
                if let Some(storage) = world.chunks.voxel_data(global).and_then(|vd| vd.player_unlockable()) {
                    self.player.set_open_storage(storage);
                    gui_controller.set_cursor_lock(self.player.is_inventory);
                    state.set_ui_interaction(self.player.is_inventory);
                } else {
                    let front = self.player.camera().front();
                    let direction = &Direction::new(front.x, front.y, front.z);
                    if let Some(block_id) = debug_block_id {
                        BLOCKS()[*block_id as usize].on_block_set(
                            &mut world, &mut self.player, &gxyz, direction);
                    } else {
                        self.player.on_right_click(&mut world, &gxyz, direction);
                    }
                }                     
            }
        }
    }
}