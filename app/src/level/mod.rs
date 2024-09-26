use std::{path::PathBuf, sync::{Arc, Mutex, Condvar, mpsc::{Sender, Receiver}}};
use graphics_engine::{mesh::Mesh, state::{State}};
use crate::{voxels::new_chunks::{ChunkCoord, GlobalCoord, WORLD_HEIGHT}, Indices};
use crate::{camera, content::Content, direction::Direction, graphic::{render::RenderResult, render_selection::render_selection}, gui::gui_controller::GuiController, input_event::{input_service::{InputService, Mouse}, KeypressState}, meshes::{Meshes, MeshesRenderInput}, my_time::Time, nalgebra_converter::Conventer, player::player::Player, recipes::{item::Item, storage::Storage}, save_load::WorldSaver, setting::Setting, threads::{save::SaveState, Threads}, unsafe_mutex::UnsafeMutex, voxels::{ray_cast::ray_cast}, world::{sun::{Color, Sun}, World}, CAMERA_FAR, CAMERA_FOV, CAMERA_NEAR};
use nalgebra_glm as glm;

pub struct Level {
    pub content: Arc<Content>,
    pub sun: Sun<9>,
    pub world: Arc<World>,
    pub player: Arc<UnsafeMutex<Player>>,
    pub threads: Option<Threads>,
    pub meshes: Meshes,
    pub render_recv: Receiver<RenderResult>,

    indices_sender: Sender<Vec<(usize, usize)>>,
    indices_recv: Receiver<Vec<(usize, usize)>>,
}

impl Level {
    pub fn new(world_name: &str, seed: u64, setting: &Setting, indices: &Indices) -> Self {
        let (render_sender, render_recv) = std::sync::mpsc::channel::<RenderResult>();
        let (indices_sender, indices_recv) = std::sync::mpsc::channel::<Vec<(usize, usize)>>();
        let mut path = PathBuf::from("./data/worlds/");
        path.push(world_name);
        let content = Arc::new(Content::new(indices, path.as_path()));
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
        let chunk_position: ChunkCoord = GlobalCoord::from(player.position().tuple()).into();
        let ox = chunk_position.x - setting.render_radius as i32;
        let oz = chunk_position.z - setting.render_radius as i32;
        let world = Arc::new(
            World::new(Arc::clone(&content), seed, render_diameter, WORLD_HEIGHT as i32, render_diameter, ox, 0, oz));
        let save_condvar = Arc::new((Mutex::new(SaveState::Unsaved), Condvar::new()));
        
        let player = Arc::new(UnsafeMutex::new(player));
        let threads = Some(Threads::new(
            content.clone(),
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
            meshes: Meshes::new(Arc::clone(&content)),
            content,
            player,
            sun,
            threads,
            world,
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
    ) -> Vec<Arc<Mesh>> {
        let mut player = unsafe {self.player.lock_unsafe()}.unwrap();
        let is_cursor = gui_controller.is_cursor();
        player.handle_input(input, time.delta(), is_cursor);
        player.inventory().lock().unwrap().update_recipe();

        let ChunkCoord{x: px, z: pz, ..} = ChunkCoord::from(GlobalCoord::from(player.position().tuple()));
        if ((px-render_radius as i32 - self.world.chunks.ox()).abs() > 2 ||
            (pz-render_radius as i32 - self.world.chunks.oz()).abs() > 2) &&
            !self.world.chunks.is_translate()
        {
            let sender = self.indices_sender.clone();
            let need_translate = self.meshes.need_translate.clone();
            self.world.chunks.set_translate(true);
            let world = self.world.clone();
            // tokio::spawn(async move {
            //     *need_translate.lock().unwrap() += 1;
            //     // let vec = world.chunks.translate(px-render_radius as i32, pz-render_radius as i32);
            //     world.chunks.set_translate(false);
            //     drop(world);
            //     let _ = sender.send(vec);
            // });
        }

        state.selection_vertex_buffer = None;
        let result = ray_cast(&self.world.chunks, &player.position().array(),
            &player.camera().front_array(), 10.0);

        if let Some(result) = result {
            let (global, voxel, norm) = (result.0, result.1, result.2);
            let global: GlobalCoord = global.into();
            let voxel_id = voxel.map_or(0, |v| v.id) as usize;

            if voxel_id != 0 {
                let min = self.content.blocks[voxel_id].min_point() + global.into();
                let max = self.content.blocks[voxel_id].max_point() + global.into();
                state.selection_vertex_buffer =
                    Some(render_selection(
                        state.device(),
                        &min.into(),
                        &max.into()
                    ));
            }

            let front = player.camera().front();
            let direction = Direction::new(front.x, front.y, front.z);

            if input.is_mouse(&Mouse::Left, KeypressState::AnyPress) && !is_cursor {
                self.content.blocks[voxel_id].on_block_break(&self.world, &mut player, &global, &direction);
            } else if input.is_mouse(&Mouse::Right, KeypressState::AnyJustPress) && !is_cursor {
                let gxyz = global + norm.tuple().into();
                let storage = self.world.chunks.master_live_voxel(global).and_then(|vd| vd.live_voxel.player_unlockable());
                if let Some(storage) = storage {
                    println!("{:?}", storage);
                    player.set_open_storage(storage);
                    gui_controller.set_cursor_lock(player.is_inventory);
                    state.set_ui_interaction(player.is_inventory);
                } else if let Some(block_id) = debug_block_id {
                    self.content.blocks[*block_id as usize].on_block_set(
                        &self.world, &mut player, &gxyz, &direction);
                } else {
                    player.on_right_click(&self.world, &gxyz, &direction, &self.content);
                }                 
            }
        }

        // let indices = frustum(
        //     &self.world.chunks,
        //     &player.camera().new_frustum(state.size.width as f32/state.size.height as f32));
        let v: Vec<ChunkCoord> = unsafe{ &*self.world.chunks.chunks.get()}.keys().cloned().collect();
        self.meshes.update_transforms_buffer(state, &self.world, &v);

        // if let Ok(indices) = self.indices_recv.try_recv() {
        //     self.meshes.translate(&indices);
        //     self.meshes.sub_need_translate();
        // }

            while let Ok(result) = self.render_recv.try_recv() {
                println!("Work!");
                    self.meshes.render(MeshesRenderInput {
                        state: &state,
                        render_result: result,
                    }, 0);
            }

        self.meshes.meshes().values().cloned().collect()
    }
}


impl Drop for Level {
    fn drop(&mut self) {
        let Some(threads) = self.threads.take() else {return};
        threads.finalize();
        println!("All saved!");
    }
}