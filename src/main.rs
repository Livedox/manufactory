use std::{time::Duration, rc::Rc, borrow::Borrow, collections::HashMap};

use camera::frustum::Frustum;
use direction::Direction;
use graphic::render_selection::render_selection;
use gui::gui_controller::GuiController;
use input_event::KeypressState;
use player::player::Player;
use recipes::{storage::Storage, item::Item};
use world::{World, global_xyz::GlobalXYZ, sun::{Sun, Color}};
use crate::voxels::chunk::HALF_CHUNK_SIZE;
use voxels::{chunks::{Chunks, WORLD_HEIGHT}, chunk::CHUNK_SIZE, block::{blocks::BLOCKS, block_type::BlockType}};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use itertools::Itertools;

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

pub fn frustum(chunks: &mut Chunks, frustum: &Frustum) -> Vec<usize> {
    let mut indices = vec![];
    for (i, c) in chunks.chunks.iter().enumerate() {
        let Some(c) = c else {continue};

        let x = c.xyz.0 as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let y = c.xyz.1 as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let z = c.xyz.2 as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        if frustum.is_cube_in(&glm::vec3(x, y, z), HALF_CHUNK_SIZE as f32) {
            indices.push(i);
        }
    }
    indices
}

pub fn main() {
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
    let window = Rc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let mut block_id = 4;

    let mut camera = camera::camera_controller::CameraController::new(glm::vec3(0.0, 0.0, 0.0), 1.2, 0.1, 1000.0);
    let mut meshes = meshes::Meshes::new();
    let mut input = input_event::input_service::InputService::new();
    let mut time = my_time::Time::new();

    let mut state = state::State::new(window.clone(), &camera.proj_view(400.0, 400.0).into());
    let mut gui_controller = GuiController::new(window, state.texture_atlas.clone());

    let mut player = Player::new();
    let binding = player.inventory();
    let mut inventory = binding.borrow_mut();
    _ = inventory.add_by_index(&Item::new(0, 100), 10);
    _ = inventory.add_by_index(&Item::new(1, 100), 11);
    _ = inventory.add_by_index(&Item::new(2, 100), 12);
    _ = inventory.add_by_index(&Item::new(3, 100), 13);
    drop(inventory);

    let mut voxel_renderer = graphic::render::VoxelRenderer {};
    let mut world = World::new(3, WORLD_HEIGHT as i32, 3);
    loop { if !world.chunks.load_visible() {break;} };
    world.chunks.set(0, 0, 0, 0, None);
    world.chunks.set(10, 10, 10, 0, None);
    world.build_sky_light();
    
    loop {
        let index = world.chunks.get_nearest_chunk_index();
        if let Some(index) = index {
            meshes.render(state.device(), &state.animated_model_layout, &mut voxel_renderer, &mut world.chunks, index, &state.animated_models);
        } else {
            break;
        }
    }
    

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
                time.update();
                camera.update(&input, time.delta(), gui_controller.is_cursor());
                state.update(&camera.proj_view(state.size.width as f32, state.size.height as f32).into(), &time);
                gui_controller.update_cursor_lock();
                if input.is_key(&Key::E, KeypressState::AnyJustPress) {
                    gui_controller.set_cursor_lock(!gui_controller.is_cursor());
                    if !gui_controller.toggle_inventory() {player.open_storage = None};
                }

                let indices = frustum(&mut world.chunks, &camera.new_frustum(state.size.width as f32/state.size.height as f32));

                let chunks_ptr = &mut world.chunks as *mut Chunks;
                world.chunks.chunks.iter_mut().enumerate().for_each(|(index, chunk)| {
                    let mut inst: Vec<u8> = vec![];
                    let Some(chunk) = chunk else { return };
                    let mut animated_models: HashMap<String, Vec<f32>> = HashMap::new();
                    chunk.voxels_data.iter_mut().sorted_by_key(|data| {data.0}).for_each(|data| {
                        data.1.update(chunks_ptr);
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
                    if let Some(mesh) = &mut meshes.mut_meshes()[index] {
                        let Some(buffer) = &mesh.transformation_matrices_buffer else {return};
                        state.queue().write_buffer(buffer, 0, inst.as_slice());
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
                        let chunk_coords = Chunks::chunk_coords(x, y, z);
                        let local_coords = Chunks::local_coords(x, y, z);
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

                        if input.is_mouse(&Mouse::Left, KeypressState::AnyJustPress) && !gui_controller.is_cursor() {
                            BLOCKS()[voxel_id as usize].on_block_break(&mut world, &mut player, &(x, y, z).into());
                        } else if input.is_mouse(&Mouse::Right, KeypressState::AnyJustPress) && !gui_controller.is_cursor() {
                            let gxyz = GlobalXYZ(x+norm.x as i32, y+norm.y as i32, z+norm.z as i32);
                            if voxel_id == 13 || voxel_id == 14 || voxel_id == 1 || voxel_id == 16 {
                                let chunk_coords = Chunks::chunk_coords(x, y, z);
                                let local_coords = Chunks::local_coords(x, y, z);
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
                }
                loop {
                    let index = world.chunks.get_nearest_chunk_index();
                    if let Some(index) = index {
                        meshes.render(state.device(), &state.animated_model_layout, &mut voxel_renderer, &mut world.chunks, index, &state.animated_models);
                    } else {
                        break;
                    }
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
