use std::{collections::{HashMap, VecDeque}, future::IntoFuture, hash::Hash, os::raw, path::Path, sync::{atomic::AtomicBool, Arc}, time::{Duration, Instant}};
use camera::frustum::Frustum;

use client_engine::ClientEngine;
use coords::{global_coord::GlobalCoord, local_coord::LocalCoord};
use graphics_engine::{constants::{BLOCK_MIPMAP_COUNT, BLOCK_TEXTURE_SIZE}, player_mesh::PlayerMesh, raw_texture::RawTexture, resources::raw_resources::{Atlas, Blocks, RawResources}, state::{self, State}};

use gui::gui_controller::GuiController;
use input_event::{input_service::InputService, KeypressState};
use level::Level;

use server_engine::ServerEngine;
use socket::client;
use unsafe_mutex::UnsafeMutex;
use world::{loader::WorldLoader};
use crate::{content_loader::{indices::{load_animated_models, load_blocks_textures, load_models, GamePath, Indices}, ContentLoader}, save_load::Save, voxels::{block::block_test::test_serde_block}};
use voxels::{live_voxels::{BoxDesiarializeLiveVoxel, BoxNewLiveVoxel, DesiarializeLiveVoxel, NewLiveVoxel}, chunks::{Chunks}};

use winit::{
    dpi::PhysicalSize, event::*, event_loop::{EventLoop, EventLoopWindowTarget}, window::{Fullscreen, WindowBuilder}
};
use itertools::{iproduct, Itertools};

use crate::{input_event::input_service::{Key}, my_time::Timer};
use nalgebra_glm as glm;
pub use graphics_engine;

pub mod content_loader;
pub mod coords;
pub mod input_event;
pub mod my_time;
pub mod voxels;
pub mod graphic;
pub mod light;
pub mod meshes;
pub mod camera;
pub mod gui;
pub mod recipes;
pub mod player;
pub mod direction;
pub mod world;
pub mod macros;
pub mod threads;
pub mod unsafe_mutex;
pub mod save_load;
pub mod bytes;
pub mod setting;
pub mod level;
pub mod nalgebra_converter;
pub mod content;
pub mod socket;
pub mod server_engine;
pub mod common;
pub mod client_engine;

const _GAME_VERSION: u32 = 1;

pub struct Registrator {
    pub c: HashMap<String, NewLiveVoxel>,
    pub from_bytes: HashMap<String, DesiarializeLiveVoxel>,
}

const CAMERA_FOV: f32 = 1.2;
const CAMERA_NEAR: f32 = 0.1;
const CAMERA_FAR: f32 = 1000.0;

pub fn load_raw_resources() -> (RawResources, Indices) {
    let (blocks_indices, blocks, blocks_count) = load_blocks_textures(&[GamePath {
        path: "./res/game/assets/blocks/".into(),
        prefix: None
    }]);
    let (models_indices, models) = load_models(&["./res/game/models"], &["./res/game/assets/models"]);
    let (animated_models_indices, animated_models) = load_animated_models(&["./res/game/animated_models"],
        &["./res/game/assets/models"]);

    let img = image::open("./res/game/assets/items/items.png").expect("./res/game/assets/items/items.png");
    let player_texture = image::open("./res/game/player.png").expect("./res/game/player.png");
    let (width, height) = (img.width(), img.height());
    if width != height { panic!("Use square textures") }


    let indices = Indices {
        block: blocks_indices,
        models: models_indices,
        animated_models: animated_models_indices,
    };

    let raw_resources = RawResources {
        blocks: Blocks {
            data: blocks,
            count: blocks_count,
        },
        atlas: Atlas {
            data: img.as_bytes().into(),
            size: width,
        },
        player: RawTexture {
            width: player_texture.width(),
            height: player_texture.height(),
            data: player_texture.as_bytes().to_vec(),
        },
        models,
        animated_models,
    };

    (raw_resources, indices)
}

// pub fn frustum(chunks: &Chunks, frustum: &Frustum) -> Vec<usize> {
//     // UPDATE
//     // This function could be much faster
//     let mut indices: Vec<usize> = vec![];
//     for (cy, cz, cx) in iproduct!(0..chunks.height, 0..chunks.depth, 0..chunks.width) {
//         let Some(c) = chunks.local_chunk(ChunkCoord::new(cx, cy, cz)) else {continue};

//         let x = c.xyz.x as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
//         let y = c.xyz.y as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
//         let z = c.xyz.z as f32 * CHUNK_SIZE as f32 + HALF_CHUNK_SIZE as f32;
//         if frustum.is_cube_in(&glm::vec3(x, y, z), HALF_CHUNK_SIZE as f32) {
//             indices.push(ChunkCoord::new(cx, cy, cz).index_without_offset(chunks.width, chunks.depth));
//         }
//     }
//     indices
// }
pub fn run_server() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut server = runtime.block_on(ServerEngine::start());
    let h = std::thread::spawn(move || {
        loop {
            server.tick();
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    h.join().unwrap();
}

pub async fn run() {
    let mut world_loader = WorldLoader::new(Path::new("./data/worlds/"));
    println!("acd");
    let mut client_engine = ClientEngine::start().await;
    println!("acsddxdd");
    let (raw_resources, indices) = load_raw_resources();
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
        &setting.graphic,
        raw_resources).await;
    let mut gui_controller = GuiController::new(window, state.resources().clone_atlas());
    // load_complex_object("transport_belt.json", &state.indices);
    let mut timer_16ms = Timer::new(Duration::from_millis(16));
    let mut fps = Instant::now();
    let mut fps_queue = VecDeque::from([0.0; 10]);
    let mut redraw = |target: &EventLoopWindowTarget<()>, input: &mut InputService, state: &mut State| {
        if exit_level {
            level = None;
            exit_level = false;
        };

        let mut debug_data = String::new();
        client_engine.player().handle_input(input, time.delta(), false);
        let mesh_vec = if let Some(level) = &mut level {
            let result = level.update(
                &input,
                &time,
                state,
                &mut gui_controller,
                &mut debug_block_id,
                setting.render_radius,
                &client_engine.player()
            );
            let player = client_engine.player();
            debug_data += &format!("{:?}", player.camera().position_tuple());
            state.update_camera(&player.camera().proj_view(state.size.width as f32, state.size.height as f32).into());
            let (sun, sky) = level.sun.sun_sky();
            state.set_sun_color(sun.into());
            state.set_clear_color(sky.into());
            // println!("Chunks: {}", unsafe {&*level.world.chunks.chunks.get()}.len());
            if input.is_key(&Key::KeyE, KeypressState::AnyJustPress) {
                gui_controller.set_cursor_lock(player.is_inventory);
                state.set_ui_interaction(player.is_inventory);
            }
            result
        } else {vec![]};
        client_engine.tick();
        
        time.update();
        state.update_time(time.current());

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

        if input.is_key(&Key::F6, KeypressState::AnyJustPress) {
            println!("{:?}", level.as_ref().unwrap().world.chunks.voxel_global(GlobalCoord::new(0, 0, 0)));
            println!("{:?}", level.as_ref().unwrap().world.chunks.voxel_global(GlobalCoord::new(1, 1, 1)));
            println!("{:?}", level.as_ref().unwrap().world.chunks.voxel_global(GlobalCoord::new(33, 1, 33)));
            for chunk in unsafe {&*level.as_ref().unwrap().world.chunks.chunks.get()}.values() {
                println!("{:?}", chunk.coord);
                println!("{:?}", chunk.voxels().get(LocalCoord::new(0, 0, 0)));
            }
        }

        if input.is_key(&Key::Escape, KeypressState::AnyJustPress) {
            gui_controller.toggle_menu();
            gui_controller.set_cursor_lock(gui_controller.is_menu);
            state.set_ui_interaction(gui_controller.is_menu);
        }

        let players_mesh: Vec<PlayerMesh> = client_engine.positions().into_iter()
            .map(|pp| PlayerMesh::new(&state, pp)).collect();
        match state.render(&mesh_vec, &players_mesh, |ctx| {
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
                    .draw_main_screen(ctx, target, &mut world_loader, &mut setting, &mut level, &indices);
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
    };

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
                    WindowEvent::RedrawRequested => { redraw(target, &mut input, &mut state) }
                    _ => {}
                }
            }
            _ => {}
        }
    }).expect("Failed to run event loop!");
}
