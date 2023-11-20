use std::{time::{Duration, Instant}, rc::Rc, borrow::Borrow, collections::HashMap, sync::{Arc, mpsc::channel, Mutex}};

use camera::frustum::Frustum;
use direction::Direction;
use graphic::{render_selection::render_selection, render::RenderResult};
use gui::gui_controller::GuiController;
use input_event::KeypressState;
use meshes::MeshesRenderInput;
use player::player::Player;
use recipes::{storage::Storage, item::Item};
use unsafe_renderer::UnsafeRenderer;
use unsafe_renderer_test::UnsafeRendererTest;
use unsafe_voxel_data_updater::spawn_unsafe_voxel_data_updater;
use world::{World, global_coords::GlobalCoords, sun::{Sun, Color}};
use world_loader::{WorldLoader};
use world_loader_test::WorldLoaderTest;
use crate::{voxels::chunk::HALF_CHUNK_SIZE, world::{global_coords, chunk_coords::ChunkCoords, local_coords::LocalCoords}};
use voxels::{chunks::{Chunks, WORLD_HEIGHT}, chunk::CHUNK_SIZE, block::{blocks::BLOCKS, block_type::BlockType}};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use itertools::{Itertools, iproduct};

use crate::{input_event::input_service::{Key, Mouse}, voxels::ray_cast, my_time::Timer};
use nalgebra_glm as glm;

mod texture;
mod state;
mod input_event;
mod my_time;
mod voxels;
mod graphic;
mod light;
mod vertices;
mod meshes;
mod camera;
mod pipelines;
mod gui;
mod recipes;
mod player;
mod models;
mod direction;
mod world;
mod world_loader;
mod unsafe_renderer;
mod unsafe_renderer_test;
mod unsafe_voxel_data_updater;
mod world_loader_test;


pub fn frustum(chunks: &mut Chunks, frustum: &Frustum, pos: &[f32; 3]) -> Vec<usize> {
    // UPDATE
    // This function could be much faster
    let mut indices: Vec<ChunkCoords> = vec![];
    let pos: ChunkCoords = GlobalCoords(pos[0] as i32, pos[1] as i32, pos[2] as i32).into();
    for (cy, cz, cx) in iproduct!(0..chunks.height, 0..chunks.depth, 0..chunks.width) {
        // let Some(c) = chunks.chunk((cx, cy, cz)) else {continue};

        let x = cx as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let y = cy as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let z = cz as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        if frustum.is_cube_in(&glm::vec3(x, y, z), HALF_CHUNK_SIZE as f32) {
            indices.push(ChunkCoords(cx, cy, cz));
        }
    }
    indices.sort_by(|a, b| {
        (a.0.abs() - pos.0 + a.1.abs() - pos.1 + a.2.abs() - pos.2)
            .cmp(&(b.0.abs() - pos.0 + b.1.abs() - pos.1 + b.2.abs() - pos.2))
    });
    indices.into_iter().map(|a| a.index(chunks.depth, chunks.width)).collect_vec()
}

pub fn main() {
    let mut render_result: Arc<Mutex<Option<RenderResult>>> = Arc::new(Mutex::new(None));
    let mut chunk_indices = Arc::new(Mutex::new(Vec::<usize>::new()));
    let sun = Sun::new(
        60,
        [0, 50, 60, 230, 240, 290, 300, 490, 500],
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
    let mut debug_data = String::new();

    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let mut block_id = 4;

    let mut camera = camera::camera_controller::CameraController::new(glm::vec3(0.0, 0.0, 0.0), 1.2, 0.1, 1000.0);
    let mut meshes = meshes::Meshes::new();
    let mut input = input_event::input_service::InputService::new();
    let mut time = my_time::Time::new();

    let mut state = state::State::new(window.clone(), &camera.proj_view(400.0, 400.0).into());
    let mut gui_controller = GuiController::new(window, state.texture_atlas.clone());

    let mut player = Player::new();
    let binding = player.inventory();
    let mut inventory = binding.lock().unwrap();
    _ = inventory.add_by_index(&Item::new(0, 100), 10);
    _ = inventory.add_by_index(&Item::new(1, 100), 11);
    _ = inventory.add_by_index(&Item::new(2, 100), 12);
    _ = inventory.add_by_index(&Item::new(3, 100), 13);
    drop(inventory);

    let player_coords = Arc::new(Mutex::new(camera.position_tuple()));
    // let world_loader = WorldLoader::new();
    let world = Arc::new(Mutex::new(World::new(30, WORLD_HEIGHT as i32, 30, 0, 0, 0)));
    let world_loader_test = WorldLoaderTest::new((&mut *world.lock().unwrap()) as *mut World, player_coords.clone());
    // world.lock().unwrap().load_chunks(&world_loader);
    
    // let renderer = UnsafeRenderer::new((&*world.lock().unwrap()) as *const World);
    // let renderer = UnsafeRendererTest::new((&mut *world.lock().unwrap()) as *mut World, chunk_indices.clone());
    let renderer = UnsafeRendererTest::new_test(
        (&mut *world.lock().unwrap()) as *mut World,
        chunk_indices.clone(),
        render_result.clone());

    spawn_unsafe_voxel_data_updater((&mut ((*world.lock().unwrap()).chunks)) as *mut Chunks);

    let mut timer_1s = Timer::new(Duration::from_secs(1));
    let mut timer_16ms = Timer::new(Duration::from_millis(16));

    let mut fps = 0;
    event_loop.run(move |event, _, control_flow| {
        state.egui_platform.handle_event(&event);
        input.process_events(&event);
        if timer_16ms.check() {
            input.update_delta_mouse();
        }
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let indices;
                // let indices_test;
                {let mut guard = world.lock().unwrap();
                let mut world: &mut World = &mut *guard;
                time.update();
                camera.update(&input, time.delta(), gui_controller.is_cursor());
                indices = frustum(
                    &mut world.chunks,
                    &camera.new_frustum(state.size.width as f32/state.size.height as f32),
                    &camera.position_array());

                if let Ok(mut lock) = chunk_indices.try_lock() {
                    *lock = indices.clone();
                }
                if let Ok(mut lock) = player_coords.try_lock() {
                    *lock = camera.position_tuple();
                }

                state.update(&camera.proj_view(state.size.width as f32, state.size.height as f32).into(), &time);
                gui_controller.update_cursor_lock();
                if input.is_key(&Key::E, KeypressState::AnyJustPress) {
                    gui_controller.set_cursor_lock(!gui_controller.is_cursor());
                    if !gui_controller.toggle_inventory() {player.open_storage = None};
                }

                indices.iter().for_each(|index| {
                    let Some(Some(chunk)) = world.chunks.chunks.get_mut(*index) else { return };

                    let mut inst: Vec<u8> = vec![];
                    let mut animated_models: HashMap<String, Vec<f32>> = HashMap::new();
                    chunk.voxels_data.iter_mut().sorted_by_key(|data| {data.0}).for_each(|data| {
                        let Some(progress) = data.1.additionally.as_ref().borrow().animation_progress() else {return};
                        let block_type = &BLOCKS()[data.1.id as usize].block_type();
                        if let BlockType::AnimatedModel {name} = block_type {
                            if let Some(animated_model) = animated_models.get_mut(name) {
                                animated_model.push(progress);
                            } else {
                                animated_models.insert(name.to_string(), vec![progress]);
                            }
                        }
                    });
                    animated_models.iter().sorted_by_key(|(name, _)| *name).for_each(|(name, progress_vec)| {
                        let model = state.animated_models.get(name).unwrap();
                        progress_vec.iter().for_each(|progress| {
                            inst.extend(model.calculate_bytes_transforms(None, *progress));
                        });
                    });
                    if let Some(Some(mesh)) = &mut meshes.mut_meshes().get(*index) {
                        let Some(buffer) = &mesh.transformation_matrices_buffer else {return};
                        if buffer.size() >= inst.len() as u64 {
                            state.queue().write_buffer(buffer, 0, inst.as_slice());
                        }
                    }
                });

                fps += 1;
                if timer_1s.check() {
                    println!("fps: {}", fps);
                    fps = 0;
                }

                if input.wheel() < 0 {
                    player.active_slot += 1;
                    if player.active_slot > 9 {player.active_slot = 0}
                }
                
                if input.wheel() > 0 {
                    if player.active_slot == 0 {
                        player.active_slot = 9
                    } else {
                        player.active_slot -= 1;
                    }
                }
                
                [Key::Key1, Key::Key2, Key::Key3, Key::Key4, Key::Key5,
                    Key::Key6, Key::Key7, Key::Key8, Key::Key9, Key::Key0]
                    .iter().enumerate().for_each(|(i, key)| {
                        if input.is_key(key, KeypressState::AnyPress) {
                            player.active_slot = i;
                        }
                    });

                {
                    let result = ray_cast::ray_cast(&world.chunks, &camera.position_array(), &camera.front_array(), 10.0);
                    if let Some(result) = result {
                        let (x, y, z, voxel, norm) = ((result.0) as i32, (result.1) as i32, (result.2) as i32, result.3, result.4);
                        let global_coords: GlobalCoords = (x, y, z).into();
                        let chunk_coords: ChunkCoords = global_coords.into();
                        let local_coords: LocalCoords = global_coords.into();
                        debug_data = format!("{:?} {:?}", result.3, world.chunks.chunk(chunk_coords).and_then(|c| c.voxel_data(local_coords)));
                        let voxel_id = voxel.map_or(0, |v| v.id);

                        if voxel_id != 0 {
                            let min_point = BLOCKS()[voxel_id as usize].min_point();
                            let max_point = BLOCKS()[voxel_id as usize].max_point();
                            state.selection_vertex_buffer =
                                Some(render_selection(
                                    state.device(),
                                    &[min_point.0 + x as f32, min_point.1 + y as f32, min_point.2 + z as f32],
                                    &[max_point.0 + x as f32, max_point.1 + y as f32, max_point.2 + z as f32]
                                ));
                        } else {
                            state.selection_vertex_buffer = None;
                        }

                        if input.is_mouse(&Mouse::Left, KeypressState::AnyPress) && !gui_controller.is_cursor() {
                            BLOCKS()[voxel_id as usize].on_block_break(&mut world, &mut player, &(x, y, z).into());
                        } else if input.is_mouse(&Mouse::Right, KeypressState::AnyJustPress) && !gui_controller.is_cursor() {
                            let gxyz = GlobalCoords(x+norm.x as i32, y+norm.y as i32, z+norm.z as i32);
                            if voxel_id == 13 || voxel_id == 14 || voxel_id == 1 || voxel_id == 16 {
                                let chunk = world.chunks.mut_chunk(chunk_coords);
                                let voxel_data = chunk.unwrap().mut_voxel_data(local_coords);
                                if let Some(storage) = voxel_data.and_then(|vd| vd.player_unlockable_storage()) {
                                    gui_controller.set_inventory(true);
                                    player.open_storage = Some(storage);
                                }
                            } else {
                                let front = camera.front();
                                player.on_right_click(&mut world, &gxyz, &Direction::new(front.x, front.y, front.z));
                            }                     
                        }
                    } else {
                        state.selection_vertex_buffer = None;
                    }
                }}

                if let Some(render_result) = render_result.lock().unwrap().take() {
                    meshes.render(MeshesRenderInput {
                        device: state.device(),
                        animated_model_layout: &state.animated_model_layout,
                        all_animated_models: &state.animated_models,
                        render_result: render_result,
                    });
                }           

                match state.render(&indices, &sun, &mut player, &gui_controller, &meshes, &time, &mut block_id, &debug_data) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(wgpu::SurfaceError::Timeout) => eprintln!("Surface timeout"),
                }
                input.update();
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
