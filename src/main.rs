use std::{time::{Duration, Instant}, rc::Rc, borrow::Borrow, collections::HashMap, sync::{Arc, mpsc::channel, Mutex}};

use camera::frustum::Frustum;
use direction::Direction;
use graphic::{render_selection::render_selection, render::RenderResult};
use gui::gui_controller::GuiController;
use input_event::KeypressState;
use meshes::{MeshesRenderInput, Meshes};
use player::player::Player;
use recipes::{storage::Storage, item::Item};
use state::State;
use world::{World, global_coords::GlobalCoords, sun::{Sun, Color}, SyncUnsafeWorldCell};
use crate::{voxels::chunk::HALF_CHUNK_SIZE, world::{global_coords, chunk_coords::ChunkCoords, local_coords::LocalCoords}};
use voxels::{chunks::{Chunks, WORLD_HEIGHT}, chunk::CHUNK_SIZE, block::{blocks::BLOCKS, block_type::BlockType}};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Fullscreen}, dpi::{PhysicalSize, LogicalSize}, monitor::VideoMode,
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
mod macros;
mod threads;


pub fn frustum(chunks: &mut Chunks, frustum: &Frustum) -> Vec<usize> {
    // UPDATE
    // This function could be much faster
    let mut indices: Vec<usize> = vec![];
    for (cy, cz, cx) in iproduct!(0..chunks.height, 0..chunks.depth, 0..chunks.width) {
        // let Some(c) = chunks.chunk((cx, cy, cz)) else {continue};

        let x = cx as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let y = cy as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let z = cz as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        if true || frustum.is_cube_in(&glm::vec3(x, y, z), HALF_CHUNK_SIZE as f32) {
            indices.push(ChunkCoords(cx, cy, cz).chunk_index(&chunks));
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
    let mut debug_block_id = None;
    let mut debug_data = String::new();


    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Manufactory")
        .with_inner_size(PhysicalSize::new(1150u32, 700u32))
        .build(&event_loop)
        .unwrap());

    let mut camera = camera::camera_controller::CameraController::new(glm::vec3(0.0, 20.0, 0.0), 1.2, 0.1, 1000.0);
    let mut meshes = meshes::Meshes::new();
    let mut input = input_event::input_service::InputService::new();
    let mut time = my_time::Time::new();
    let window_size = window.inner_size();
    let mut state = state::State::new(
        window.clone(), &camera.proj_view(window_size.width as f32, window_size.height as f32).into());
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
    let mut world = Arc::new(SyncUnsafeWorldCell::new(World::new(1, WORLD_HEIGHT as i32, 1, 0, 0, 0)));


    let render_result: Arc<Mutex<Option<RenderResult>>> = Arc::new(Mutex::new(None));
    threads::world_loader::spawn(world.clone(), player_coords.clone());
    threads::renderer::spawn(world.clone(), player_coords.clone(), render_result.clone());
    threads::voxel_data_updater::spawn(world.clone());

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
                let world = world.get_mut();
                time.update();
                camera.update(&input, time.delta(), gui_controller.is_cursor());
                let indices = frustum(
                    &mut world.chunks,
                    &camera.new_frustum(state.size.width as f32/state.size.height as f32));
                *player_coords.lock().unwrap() = camera.position_tuple();
                state.update(&camera.proj_view(state.size.width as f32, state.size.height as f32).into(), &time);
                gui_controller.update_cursor_lock();
                meshes.update_transforms_buffer(&state, &world, &indices);

                let c: ChunkCoords = GlobalCoords::from(camera.position_tuple()).into();
                if c.0 != world.chunks.ox || c.2 != world.chunks.oz {
                    let indices = world.chunks.translate(c.0, c.2);
                    meshes.translate(&indices);
                    println!("Count: {}", world.chunks.chunks.iter().map(|c| c.is_some() as usize).sum::<usize>());
                    println!("Count meshes: {}", meshes.meshes().iter().map(|c| c.is_some() as usize).sum::<usize>());
                }

                fps += 1;
                if timer_1s.check() {
                    println!("{:?}", world.chunks.chunk((0, 0, -1)).map(|c| c.voxel(LocalCoords(0, 0, 0))));
                    fps = 0;
                }

                if input.is_key(&Key::E, KeypressState::AnyJustPress) {
                    gui_controller.set_cursor_lock(!gui_controller.is_cursor());
                    if !gui_controller.toggle_inventory() {player.open_storage = None};
                }

                if input.is_key(&Key::F1, KeypressState::AnyJustPress) {
                    gui_controller.toggle_ui();
                }
                
                if input.is_key(&Key::F11, KeypressState::AnyJustPress) {
                    let window = state.window();
                    if window.fullscreen().is_some() {
                        window.set_fullscreen(None);
                    } else {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
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

                    if input.is_mouse(&Mouse::Left, KeypressState::AnyJustPress) && !gui_controller.is_cursor() {
                        BLOCKS()[voxel_id as usize].on_block_break(world, &mut player, &(x, y, z).into());
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
                            if let Some(block_id) = debug_block_id {
                                BLOCKS()[block_id as usize].on_block_set(
                                    world, &mut player, &gxyz, &Direction::new(front.x, front.y, front.z));
                            } else {
                                player.on_right_click(world, &gxyz, &Direction::new(front.x, front.y, front.z));
                            }
                        }                     
                    }
                } else {
                    state.selection_vertex_buffer = None;
                }

                let render_result = render_result.lock().unwrap().take();
                if let Some(render_result) = render_result {
                    meshes.render(MeshesRenderInput {
                        device: state.device(),
                        animated_model_layout: &state.animated_model_layout,
                        all_animated_models: &state.animated_models,
                        render_result: render_result,
                    });
                }           

                match state.render(&indices, &sun, &mut player, &gui_controller, &meshes, &time, &mut debug_block_id, &debug_data) {
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
