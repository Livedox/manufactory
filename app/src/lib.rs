use std::{collections::{HashMap, VecDeque}, future::IntoFuture, hash::Hash, path::Path, sync::{atomic::AtomicBool, Arc}, time::{Duration, Instant}};
use camera::frustum::Frustum;

use coords::chunk_coord::ChunkCoord;
use graphics_engine::{constants::{BLOCK_MIPMAP_COUNT, BLOCK_TEXTURE_SIZE}, state};

use gui::gui_controller::GuiController;
use image::imageops::FilterType;
use input_event::KeypressState;
use level::Level;

use unsafe_mutex::UnsafeMutex;
use world::{loader::WorldLoader};
use crate::{save_load::Save, voxels::{block::block_test::test_serde_block, chunk::HALF_CHUNK_SIZE}};
use voxels::{chunk::CHUNK_SIZE, chunks::Chunks, live_voxels::{BoxDesiarializeLiveVoxel, BoxNewLiveVoxel, DesiarializeLiveVoxel, NewLiveVoxel}};

use winit::{
    event::*,
    event_loop::{EventLoop},
    window::{WindowBuilder, Fullscreen}, dpi::PhysicalSize,
};
use itertools::{iproduct, Itertools};

use crate::{input_event::input_service::{Key}, my_time::Timer};
use nalgebra_glm as glm;
pub use graphics_engine;
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

static WORLD_EXIT: AtomicBool = AtomicBool::new(false);
const _GAME_VERSION: u32 = 1;

pub struct Registrator {
    pub c: HashMap<String, NewLiveVoxel>,
    pub from_bytes: HashMap<String, DesiarializeLiveVoxel>,
}

const CAMERA_FOV: f32 = 1.2;
const CAMERA_NEAR: f32 = 0.1;
const CAMERA_FAR: f32 = 1000.0;

pub fn load_blocks_textures(path: impl AsRef<Path>) -> (HashMap::<String, u32>, Vec<Vec<u8>>, u32) {
    let files = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();

    let mut indices = HashMap::<String, u32>::new();
    let images = files.map(|(index, file)| {
        let name = file.file_name().to_str().unwrap();
        let dot_index = name.rfind('.').unwrap();
        indices.insert(name[..dot_index].to_string(), index as u32);

        image::open(file.path()).unwrap_or_else(|_| panic!("Failed to open image on path: {:?}", file.path()))
    }).collect_vec();

    let data = (0..BLOCK_MIPMAP_COUNT).map(|mipmap| {
        let size = BLOCK_TEXTURE_SIZE / 2u32.pow(mipmap as u32);
        images.iter().flat_map(|image| {
            if mipmap == 0 {return image.to_rgba8().to_vec()};
            image.resize(size, size, FilterType::Triangle).to_rgba8().to_vec()
        }).collect_vec()
    }).collect_vec();

    (indices, data, images.len() as u32)
}

pub fn load_animated_models(
    models_path: impl AsRef<Path>,
    textures_path: impl AsRef<Path>
) -> (HashMap::<String, u32>, Vec<resources::animated_model::AnimatedModel>) {
    let files = walkdir::WalkDir::new(models_path.as_ref())
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();
    let textures_path = textures_path.as_ref();

    let mut indices = HashMap::<String, u32>::new();
    let models: Vec<resources::animated_model::AnimatedModel> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let texture_name = name.clone() + ".png";
        let src_texture = textures_path.join(texture_name);
        let model = resources::animated_model::load_animated_model(file.path(), src_texture);
        indices.insert(name, index as u32);
        model
    }).collect();

    println!("{:?}", indices);

    (indices, models)
  }

pub fn load_models(
    models_path: impl AsRef<Path>,
    textures_path: impl AsRef<Path>,
) -> (HashMap::<String, u32>, Vec<resources::model::Model>) {
    let files = walkdir::WalkDir::new(models_path.as_ref())
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();
    let textures_path = textures_path.as_ref();

    let mut indices = HashMap::<String, u32>::new();
    let models: Vec<resources::model::Model> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let texture_name = name.clone() + ".png";
        let src_texture = textures_path.join(texture_name);
        let model = resources::model::load_model(file.path(), src_texture).unwrap();
        indices.insert(name, index as u32);
        model
    }).collect();

    (indices, models)
}

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

pub struct Indices {
    pub block: HashMap<String, u32>,
    pub models: HashMap<String, u32>,
    pub animated_models: HashMap<String, u32>,
}

#[no_mangle]
pub extern "C" fn run() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(run_async());
}

pub async fn run_async() {
    println!("{:?}", Path::new("./data/").canonicalize());
    let mut world_loader = WorldLoader::new(Path::new("./data/worlds/"));
    //let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    //let file = BufReader::new(File::open("./audio/music/Kyle Gabler - Years of Work.mp3").unwrap());
    // Decode that sound file into a source
    //let source = Decoder::new(file).unwrap();
    // Play the sound directly on the device
    //let _ = stream_handle.play_raw(source.convert_samples());
    let (blocks_indices, blocks, blocks_len) = load_blocks_textures("./res/game/assets/blocks/");
    let (models_indices, models) = load_models("./res/game/models", "./res/game/assets/models");
    let (animated_models_indices, animated_models) = load_animated_models("./res/game/animated_models", "./res/game/assets/models");

    let img = image::open("./res/game/assets/items/items.png").expect("./res/game/assets/items/items.png");
    let (width, height) = (img.width(), img.height());
    if width != height { panic!("Use square textures") }


    let indices = Indices {
        block: blocks_indices,
        models: models_indices,
        animated_models: animated_models_indices,
    };

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
        blocks,
        blocks_len,
        models,
        animated_models,
        img.as_bytes(),
        width,
        &[[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]).await;
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
        
                        if input.is_key(&Key::Escape, KeypressState::AnyJustPress) {
                            gui_controller.toggle_menu();
                            gui_controller.set_cursor_lock(gui_controller.is_menu);
                            state.set_ui_interaction(gui_controller.is_menu);
                        }

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
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).expect("Failed to run event loop!");
}
