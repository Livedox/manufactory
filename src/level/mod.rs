use std::{sync::{Arc, Mutex, Condvar, mpsc::{Sender, Receiver}, RwLock}, path::PathBuf, marker::PhantomPinned};
use crate::{unsafe_mutex::UnsafeMutex, world::{World, sun::{Sun, Color}, global_coords::GlobalCoords, chunk_coords::ChunkCoords, local_coords::LocalCoords}, threads::{Threads, save::SaveState}, player::player::Player, save_load::WorldSaver, camera, recipes::{storage::Storage, item::Item}, CAMERA_FOV, CAMERA_NEAR, CAMERA_FAR, setting::Setting, voxels::{chunks::WORLD_HEIGHT, ray_cast::ray_cast, block::blocks::BLOCKS}, graphic::{render::RenderResult, render_selection::render_selection}, nalgebra_converter::Conventer, input_event::{input_service::{InputService, Mouse}, KeypressState}, my_time::Time, direction::Direction, engine::state::State, gui::gui_controller::{self, GuiController}, meshes::{Meshes, MeshesRenderInput, Mesh}, frustum};
use nalgebra_glm as glm;

pub struct Level {
    pub sun: Sun<9>,
    world_saver: Arc<WorldSaver>,
    pub world: Arc<World>,
    pub player: Arc<UnsafeMutex<Player>>,
    pub threads: Option<Threads>,
    pub meshes: Meshes,
    pub render_recv: Receiver<RenderResult>,

    indices_sender: Sender<Vec<(usize, usize)>>,
    indices_recv: Receiver<Vec<(usize, usize)>>,
}

impl Level {
    pub fn new(world_name: &str, setting: &Setting) -> Self {
        let (render_sender, render_recv) = std::sync::mpsc::channel::<RenderResult>();
        let (indices_sender, indices_recv) = std::sync::mpsc::channel::<Vec<(usize, usize)>>();
        let mut path = PathBuf::from("./data/worlds/");
        path.push(world_name);
        let world_saver = Arc::new(WorldSaver::new(path));
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
        let world = Arc::new(
            World::new(render_diameter, WORLD_HEIGHT as i32, render_diameter, ox, 0, oz));
        let save_condvar = Arc::new((Mutex::new(SaveState::Unsaved), Condvar::new()));
        
        let player = Arc::new(UnsafeMutex::new(player));
        let threads = Some(Threads::new(
            world.clone(),
            player.clone(),
            world_saver.clone(),
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
            world,
            meshes: Meshes::new(),
            render_recv,
            indices_sender,
            indices_recv
        }
    }

    pub fn update(
      &mut self,
      input: &InputService,
      time: &Time,
      state: &mut State,
      gui_controller: &mut GuiController,
      debug_block_id: &mut Option<u32>,
      render_radius: u32,
    ) -> Vec<&Mesh> {
        let mut player = unsafe {self.player.lock_unsafe()}.unwrap();
        let is_cursor = gui_controller.is_cursor();
        player.handle_input(input, time.delta(), is_cursor);
        player.inventory().lock().unwrap().update_recipe();

        let ChunkCoords(px, _, pz) = ChunkCoords::from(GlobalCoords::from(player.position().tuple()));
        if ((px-render_radius as i32 - self.world.chunks.ox()).abs() > 2 ||
            (pz-render_radius as i32 - self.world.chunks.oz()).abs() > 2) &&
            !self.world.chunks.is_translate()
        {
            let sender = self.indices_sender.clone();
            let need_translate = self.meshes.need_translate.clone();
            self.world.chunks.set_translate(true);
            let world = self.world.clone();
            tokio::spawn(async move {
                *need_translate.lock().unwrap() += 1;
                let vec = world.chunks.translate(px as i32-render_radius as i32, pz as i32-render_radius as i32);
                world.chunks.set_translate(false);
                drop(world);
                let _ = sender.send(vec);
            });
        }

        state.selection_vertex_buffer = None;
        let result = ray_cast(&self.world.chunks, &player.position().array(),
            &player.camera().front_array(), 10.0);

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
            }

            if input.is_mouse(&Mouse::Left, KeypressState::AnyJustPress) && !is_cursor {
                BLOCKS()[voxel_id as usize].on_block_break(&self.world, &mut player, &global);
            } else if input.is_mouse(&Mouse::Right, KeypressState::AnyJustPress) && !is_cursor {
                let gxyz = global + norm.tuple().into();
                let storage = self.world.chunks.voxel_data(global).and_then(|vd| vd.player_unlockable());
                if let Some(storage) = storage {
                    player.set_open_storage(storage);
                    gui_controller.set_cursor_lock(player.is_inventory);
                    state.set_ui_interaction(player.is_inventory);
                } else {
                    let front = player.camera().front();
                    let direction = &Direction::new(front.x, front.y, front.z);
                    if let Some(block_id) = debug_block_id {
                        BLOCKS()[*block_id as usize].on_block_set(
                            &self.world, &mut player, &gxyz, direction);
                    } else {
                        player.on_right_click(&self.world, &gxyz, direction);
                    }
                }                     
            }
        }

        let indices = frustum(
            &self.world.chunks,
            &player.camera().new_frustum(state.size.width as f32/state.size.height as f32));
        self.meshes.update_transforms_buffer(&state, &self.world, &indices);

        if let Ok(indices) = self.indices_recv.try_recv() {
            self.meshes.translate(&indices);
            self.meshes.sub_need_translate();
        }

        if !self.meshes.is_need_translate() {
            while let Ok(result) = self.render_recv.try_recv() {
                if self.world.chunks.is_in_area(result.xyz) {
                    let index = result.xyz.chunk_index(&self.world.chunks);
                    self.meshes.render(MeshesRenderInput {
                        device: state.device(),
                        animated_model_layout: &state.layouts.transforms_storage,
                        all_animated_models: &state.animated_models,
                        render_result: result,
                    }, index);
                }
            }
        }

        indices.iter().filter_map(|i| self.meshes.meshes().get(*i).and_then(|c| c.as_ref()))
            .collect::<Vec<&Mesh>>()
    }
}


impl Drop for Level {
    fn drop(&mut self) {
        let Some(threads) = self.threads.take() else {return};
        threads.finalize();
        println!("All saved!");
    }
}