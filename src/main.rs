use std::{time::{Duration, Instant}, sync::{Arc, Mutex, Condvar}, collections::VecDeque};
use camera::frustum::Frustum;
use direction::Direction;
use engine::state;
use graphic::{render_selection::render_selection, render::RenderResult};
use gui::gui_controller::GuiController;
use input_event::KeypressState;
use meshes::{MeshesRenderInput, Mesh};
use player::player::Player;
use recipes::{storage::Storage, item::Item};
use threads::save::SaveState;
use unsafe_mutex::UnsafeMutex;
use world::{World, global_coords::GlobalCoords, sun::{Sun, Color}};
use crate::{voxels::chunk::HALF_CHUNK_SIZE, world::{chunk_coords::ChunkCoords, local_coords::LocalCoords}, save_load::Save};
use voxels::{chunks::{Chunks, WORLD_HEIGHT}, chunk::CHUNK_SIZE, block::blocks::BLOCKS};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Fullscreen}, dpi::PhysicalSize,
};
use itertools::iproduct;

use crate::{input_event::input_service::{Key, Mouse}, voxels::ray_cast, my_time::Timer};
use nalgebra_glm as glm;

mod input_event;
mod my_time;
mod voxels;
mod graphic;
mod light;
mod meshes;
mod camera;
mod gui;
mod recipes;
mod player;
mod models;
mod direction;
mod world;
mod macros;
mod threads;
mod unsafe_mutex;
mod engine;
mod save_load;
mod bytes;

static mut WORLD_EXIT: bool = false;
const _GAME_VERSION: u32 = 1;

const RENDER_DISTANCE: i32 = 10;
const HALF_RENDER_DISTANCE: i32 = RENDER_DISTANCE / 2;

const CAMERA_FOV: f32 = 1.2;
const CAMERA_NEAR: f32 = 0.1;
const CAMERA_FAR: f32 = 1000.0;

pub fn frustum(chunks: &mut Chunks, frustum: &Frustum) -> Vec<usize> {
    // UPDATE
    // This function could be much faster
    let mut indices: Vec<usize> = vec![];
    for (cy, cz, cx) in iproduct!(0..chunks.height, 0..chunks.depth, 0..chunks.width) {
        let Some(c) = chunks.local_chunk(ChunkCoords(cx, cy, cz)) else {continue};

        let x = c.xyz.0 as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let y = c.xyz.1 as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let z = c.xyz.2 as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        if frustum.is_cube_in(&glm::vec3(x, y, z), HALF_CHUNK_SIZE as f32) {
            indices.push(ChunkCoords(cx, cy, cz).index_without_offset(chunks.width, chunks.depth));
        }
    }
    indices
}


#[tokio::main]
pub async fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<Vec<(usize, usize)>>();
    let (render_sender, render_recv) = std::sync::mpsc::channel::<RenderResult>();
    let save = Save::new("./data/worlds/debug/");
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

    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Manufactory")
        .with_inner_size(PhysicalSize::new(1150u32, 700u32))
        .build(&event_loop)
        .unwrap());

    let mut player = match save.world.player.lock().unwrap().load_player() {
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
    
    let mut meshes = meshes::Meshes::new();
    let mut input = input_event::input_service::InputService::new();
    let mut time = my_time::Time::new();
    let window_size = window.inner_size();
    let mut state = state::State::new(
        window.clone(), &player.camera().proj_view(window_size.width as f32, window_size.height as f32).into()).await;
    let mut gui_controller = GuiController::new(window, state.texture_atlas.clone());

    let player_coords = Arc::new(Mutex::new(player.camera().position_tuple()));
    let c: ChunkCoords = GlobalCoords::from(player.camera().position_tuple()).into();
    let ox = c.0 - HALF_RENDER_DISTANCE;
    let oz = c.2 - HALF_RENDER_DISTANCE;
    let world = Arc::new(UnsafeMutex::new(
        World::new(RENDER_DISTANCE, WORLD_HEIGHT as i32, RENDER_DISTANCE, ox, 0, oz)));
    let render_result: Arc<Mutex<Option<RenderResult>>> = Arc::new(Mutex::new(None));
    let save_condvar = Arc::new((Mutex::new(SaveState::Unsaved), Condvar::new()));
    
    let thread_save = threads::save::spawn(world.clone(), save.world.regions.clone(), save_condvar.clone());
    let thread_world_loader = threads::world_loader::spawn(world.clone(), save.world.regions.clone(), player_coords.clone());
    let thread_renderer = threads::renderer::spawn(world.clone(), render_sender, render_result.clone());
    let thread_voxel_data_updater = threads::voxel_data_updater::spawn(world.clone());
    
    let mut finalize = Some(move || {
        unsafe {WORLD_EXIT = true};
        let _ = thread_renderer.join();
        let _ = thread_world_loader.join();
        let _ = thread_voxel_data_updater.join();

        let (save_state, cvar) = &*save_condvar;
        *save_state.lock().unwrap() = SaveState::WorldExit;
        cvar.notify_one();
        thread_save.join().expect("Failed to terminate thread save");
    });

    let mut timer_16ms = Timer::new(Duration::from_millis(16));
    let mut fps = Instant::now();
    let mut fps_queue = VecDeque::from([0.0; 10]);
    event_loop.run(move |event, _, control_flow| {
        state.handle_event(&event);
        input.handle_event(&event);
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
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                player.handle_input(&input, time.delta(), gui_controller.is_cursor());

                let mut world_g = unsafe {world.lock_immediately()}.unwrap();
                time.update();
                let c: ChunkCoords = GlobalCoords::from(player.camera().position_tuple()).into();
                let mut debug_data = format!("{:?}", player.camera().position_tuple());
                if ((c.0-HALF_RENDER_DISTANCE - world_g.chunks.ox).abs() >= 2 || (c.2-HALF_RENDER_DISTANCE - world_g.chunks.oz).abs() >= 2) && !world_g.chunks.is_translate {
                    world_g.chunks.is_translate = true;
                    drop(world_g);
                    let w = world.clone();
                    let tx_clone = tx.clone();
                    let need_translate = meshes.need_translate.clone();
                    tokio::spawn(async move {
                        let mut world = w.lock().unwrap();
                        *need_translate.lock().unwrap() += 1;
                        let vec = world.chunks.translate(c.0-HALF_RENDER_DISTANCE, c.2-HALF_RENDER_DISTANCE);
                        world.chunks.is_translate = false;
                        drop(world);
                        let _ = tx_clone.send(vec);
                    });
                }
                let mut world_g = unsafe {world.lock_immediately()}.unwrap();
                let indices = frustum(
                    &mut world_g.chunks,
                    &player.camera().new_frustum(state.size.width as f32/state.size.height as f32));
                *player_coords.lock().unwrap() = player.camera().position_tuple();
                state.update(&player.camera().proj_view(state.size.width as f32, state.size.height as f32).into(), &time);
                gui_controller.update_cursor_lock();
                meshes.update_transforms_buffer(&state, &world_g, &indices);

                fps_queue.push_back(1.0/fps.elapsed().as_secs_f32());
                debug_data += &(fps_queue.iter().sum::<f32>() / fps_queue.len() as f32).floor().to_string();
                fps_queue.pop_front();
                fps = Instant::now();

                if input.is_key(&Key::E, KeypressState::AnyJustPress) {
                    gui_controller.set_cursor_lock(player.is_inventory);
                    state.set_ui_interaction(player.is_inventory);
                }

                if input.is_key(&Key::F1, KeypressState::AnyJustPress) {
                    gui_controller.toggle_ui();
                    state.set_crosshair(gui_controller.is_ui());
                }
                
                if input.is_key(&Key::F11, KeypressState::AnyJustPress) {
                    let window = state.window();
                    if window.fullscreen().is_some() {
                        window.set_fullscreen(None);
                    } else {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
                }

                let result = ray_cast::ray_cast(&world_g.chunks, &player.camera().position_array(), &player.camera().front_array(), 10.0);
                if let Some(result) = result {
                    let (x, y, z, voxel, norm) = ((result.0) as i32, (result.1) as i32, (result.2) as i32, result.3, result.4);
                    let global_coords: GlobalCoords = (x, y, z).into();
                    let chunk_coords: ChunkCoords = global_coords.into();
                    let local_coords: LocalCoords = global_coords.into();
                    debug_data += &format!("{:?} {:?}", result.3, world_g.chunks.chunk(chunk_coords).and_then(|c| c.voxel_data(local_coords)));
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
                        BLOCKS()[voxel_id as usize].on_block_break(&mut world_g, &mut player, &(x, y, z).into());
                    } else if input.is_mouse(&Mouse::Right, KeypressState::AnyJustPress) && !gui_controller.is_cursor() {
                        let gxyz = GlobalCoords(x+norm.x as i32, y+norm.y as i32, z+norm.z as i32);
                        if let Some(storage) = world_g.chunks.voxel_data(global_coords).and_then(|vd| vd.player_unlockable()) {
                            player.set_open_storage(storage);
                            gui_controller.set_cursor_lock(player.is_inventory);
                            state.set_ui_interaction(player.is_inventory);
                        } else {
                            let front = player.camera().front();
                            let direction = &Direction::new(front.x, front.y, front.z);
                            if let Some(block_id) = debug_block_id {
                                BLOCKS()[block_id as usize].on_block_set(
                                    &mut world_g, &mut player, &gxyz, direction);
                            } else {
                                player.on_right_click(&mut world_g, &gxyz, direction);
                            }
                        }                     
                    }
                } else {
                    state.selection_vertex_buffer = None;
                }

                if let Ok(indices) = rx.try_recv() {
                    meshes.translate(&indices);
                    meshes.sub_need_translate();
                }

                if !meshes.is_need_translate() {
                    while let Ok(result) = render_recv.try_recv() {
                        if world_g.chunks.is_in_area(result.xyz) {
                            let index = result.xyz.chunk_index(&world_g.chunks);
                            meshes.render(MeshesRenderInput {
                                device: state.device(),
                                animated_model_layout: &state.layouts.transforms_storage,
                                all_animated_models: &state.animated_models,
                                render_result: result,
                            }, index);
                        }
                    }
                }

                
                player.inventory().lock().unwrap().update_recipe();
                let (sun, sky) = sun.sun_sky();
                state.set_sun_color(sun.into());
                state.set_clear_color(sky.into());
                let mesh_vec = indices.iter().filter_map(|i| meshes.meshes().get(*i).and_then(|c| c.as_ref()))
                    .collect::<Vec<&Mesh>>();
                
                match state.render(&mesh_vec, |ctx| {
                    gui_controller
                        .draw_inventory(ctx, &mut player)
                        .draw_debug(ctx, &debug_data, &mut debug_block_id)
                        .draw_active_recieps(ctx, &mut player);
                }) {
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
            Event::LoopDestroyed => {
                finalize.take().unwrap()();
                save.world.player.lock().unwrap().save_player(&player);
                println!("All saved!");
            }
            _ => {}
        }
    });
}
