use std::{time::{Duration, Instant}, sync::{Arc, atomic::{AtomicBool}}, collections::{VecDeque}, path::Path};
use camera::frustum::Frustum;

use coords::chunk_coord::ChunkCoord;
use engine::state;

use gui::gui_controller::GuiController;
use input_event::KeypressState;
use level::Level;

use unsafe_mutex::UnsafeMutex;
use world::{loader::WorldLoader};
use crate::{engine::state::{Indices}, save_load::Save, voxels::{block::block_test::test_serde_block, chunk::HALF_CHUNK_SIZE}};
use voxels::{chunks::{Chunks}, chunk::CHUNK_SIZE};

use winit::{
    event::*,
    event_loop::{EventLoop},
    window::{WindowBuilder, Fullscreen}, dpi::PhysicalSize,
};
use itertools::iproduct;

use crate::{input_event::input_service::{Key}, my_time::Timer};
use nalgebra_glm as glm;

mod coords;
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
mod direction;
mod world;
mod macros;
mod threads;
mod unsafe_mutex;
mod engine;
mod save_load;
mod bytes;
mod setting;
mod level;
mod nalgebra_converter;
mod content;

static WORLD_EXIT: AtomicBool = AtomicBool::new(false);
const _GAME_VERSION: u32 = 1;

const CAMERA_FOV: f32 = 1.2;
const CAMERA_NEAR: f32 = 0.1;
const CAMERA_FAR: f32 = 1000.0;

pub fn frustum(chunks: &Chunks, frustum: &Frustum) -> Vec<usize> {
    // UPDATE
    // This function could be much faster
    let mut indices: Vec<usize> = vec![];
    for (cy, cz, cx) in iproduct!(0..chunks.height, 0..chunks.depth, 0..chunks.width) {
        let Some(c) = chunks.local_chunk(ChunkCoord::new(cx, cy, cz)) else {continue};

        let x = c.xyz.x as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let y = c.xyz.y as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        let z = c.xyz.z as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
        if frustum.is_cube_in(&glm::vec3(x, y, z), HALF_CHUNK_SIZE as f32) {
            indices.push(ChunkCoord::new(cx, cy, cz).index_without_offset(chunks.width, chunks.depth));
        }
    }
    indices
}


#[tokio::main]
pub async fn main() {
    println!("{:?}", Path::new("./data/").canonicalize());
    let mut world_loader = WorldLoader::new(Path::new("./data/worlds/"));
    //let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    //let file = BufReader::new(File::open("./audio/music/Kyle Gabler - Years of Work.mp3").unwrap());
    // Decode that sound file into a source
    //let source = Decoder::new(file).unwrap();
    // Play the sound directly on the device
    //let _ = stream_handle.play_raw(source.convert_samples());
    let save = Save::new("./data/worlds/debug/", "./data/");
    let mut setting = save.setting.load().unwrap_or_default();
    save.setting.save(&setting);

    let mut debug_block_id = None;

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(WindowBuilder::new()
        .with_title("Manufactory")
        .with_inner_size(PhysicalSize::new(1150u32, 700u32))
        .build(&event_loop)
        .unwrap());
        
    let mut input = input_event::input_service::InputService::new();
    let mut time = my_time::Time::new();

    let mut level: Option<Level> = None;
    let mut exit_level = false;
    let mut state = state::State::new(
        window.clone(),
        &[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        &setting.graphic).await;
    let mut gui_controller = GuiController::new(window, state.texture_atlas.clone());
    // load_complex_object("transport_belt.json", &state.indices);
    let mut timer_16ms = Timer::new(Duration::from_millis(16));
    let mut fps = Instant::now();
    let mut fps_queue = VecDeque::from([0.0; 10]);
    event_loop.run(move |event, target| {
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
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::RedrawRequested => {
                        if exit_level {
                            level = None;
                            exit_level = false;
                        };
        
                        let mut debug_data = String::new();
                        let mesh_vec = if let Some(level) = &mut level {
                            let result = level.update(
                                &input,
                                &time,
                                &mut state,
                                &mut gui_controller,
                                &mut debug_block_id,
                                setting.render_radius,
                            );
                            let player = unsafe {level.player.lock_unsafe()}.unwrap();
                            debug_data += &format!("{:?}", player.camera().position_tuple());
                            state.update_camera(&player.camera().proj_view(state.size.width as f32, state.size.height as f32).into());
                            let (sun, sky) = level.sun.sun_sky();
                            state.set_sun_color(sun.into());
                            state.set_clear_color(sky.into());
        
                            if input.is_key(&Key::KeyE, KeypressState::AnyJustPress) {
                                gui_controller.set_cursor_lock(player.is_inventory);
                                state.set_ui_interaction(player.is_inventory);
                            }
                            result
                        } else {vec![]};
                        
                        
                        time.update();
                        state.update_time(&time);
        
                        gui_controller.update_cursor_lock();
        
                        fps_queue.push_back(1.0/fps.elapsed().as_secs_f32());
                        debug_data += &(fps_queue.iter().sum::<f32>() / fps_queue.len() as f32).floor().to_string();
                        fps_queue.pop_front();
                        fps = Instant::now();
        
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
        
                        if input.is_key(&Key::Escape, KeypressState::AnyJustPress) {
                            gui_controller.toggle_menu();
                            gui_controller.set_cursor_lock(gui_controller.is_menu);
                            state.set_ui_interaction(gui_controller.is_menu);
                        }
                        
                        let indices = unsafe {&*(&state.indices as *const Indices)};
                        match state.render(&mesh_vec, |ctx| {
                            if let Some(l) = &level {
                                let mut player = unsafe {l.player.lock_unsafe()}.unwrap();
                                gui_controller
                                    .draw_inventory(ctx, &mut player)
                                    .draw_debug(ctx, &debug_data, &mut debug_block_id)
                                    .draw_active_recieps(ctx, &mut player);
        
                                drop(player);
                                gui_controller.draw_in_game_menu(ctx, &mut exit_level);
                            } else {
                                gui_controller
                                    .draw_main_screen(ctx, target, &mut world_loader, &mut setting, &mut level, indices);
                            }
        
                            gui_controller.draw_setting(ctx, &mut setting, &save.setting);
                        }) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.resize(state.size)
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(wgpu::SurfaceError::Timeout) => eprintln!("Surface timeout"),
                        }
                        input.update();
                        state.window().request_redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).expect("Failed to run event loop!");
}
