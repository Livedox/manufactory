use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use graphics_engine::player_mesh::PlayerMesh;
use graphics_engine::resources::raw_resources::RawResources;
use graphics_engine::state::{self, State};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Fullscreen, Window, WindowId};
use crate::client_engine::ClientEngine;
use crate::content_loader::indices::Indices;
use crate::coords::global_coord::GlobalCoord;
use crate::coords::local_coord::LocalCoord;
use crate::gui::gui_controller::GuiController;
use crate::input_event::input_service::{InputService, Key};
use crate::input_event::KeypressState;
use crate::level::Level;
use crate::my_time::Timer;
use crate::setting::Setting;
use crate::{input_event, load_raw_resources, my_time};
use crate::save_load::Save;
use crate::world::loader::WorldLoader;

pub struct App<'a> {
    state: Option<State<'a>>,
    setting: Setting,
    raw_resources: Option<RawResources>,
    gui_controller: Option<GuiController>,
    client_engine: ClientEngine,
    input: InputService,
    level: Option<Level>,
    time: my_time::Time,
    exit_level: bool,
    timer_16ms: Timer,
    fps: Instant,
    fps_queue: VecDeque<f32>,
    debug_block_id: Option<u32>,
    world_loader: WorldLoader,
    indices: Indices,
    save: Save,
    tick: u32,
}

impl<'a> App<'a> {
    pub async fn new() -> Self {
        let mut world_loader = WorldLoader::new(Path::new("./data/worlds/"));
        let (raw_resources, indices) = load_raw_resources();
        let save = Save::new("./data/worlds/debug/", "./data/");
        let mut setting = save.setting.load().unwrap_or_default();
        save.setting.save(&setting);

        let mut client_engine = ClientEngine::start().await;
        let mut input = input_event::input_service::InputService::new();
        let mut time = my_time::Time::new();

        let mut level: Option<Level> = None;
        let mut exit_level = false;
        let mut debug_block_id = None;
        let mut timer_16ms = Timer::new(Duration::from_millis(16));
        let mut fps = Instant::now();
        let mut fps_queue = VecDeque::from([0.0; 10]);

        Self {
            raw_resources: Some(raw_resources),
            tick: 0,
            setting,
            state: None,
            gui_controller: None,
            client_engine,
            exit_level,
            fps,
            fps_queue,
            input,
            level,
            time,
            timer_16ms,
            debug_block_id,
            save,
            world_loader,
            indices,
        }
    }
}

impl ApplicationHandler<crate::Timer> for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let win_attr = Window::default_attributes()
                .with_title("Manufactory")
                .with_inner_size(PhysicalSize::new(1150u32, 700u32));
            let window = event_loop.create_window(win_attr).unwrap();

            println!("{:?}", window.display_handle());
            println!("{:?}", window.window_handle());
            let state = pollster::block_on(state::State::new(
                window,
                &self.setting.graphic,
                self.raw_resources.take().unwrap()));
            let mut gui_controller = GuiController::new(state.window().clone(), state.resources().clone_atlas());
            self.state = Some(state);
            self.gui_controller = Some(gui_controller)
            
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let mut state = self.state.as_mut().unwrap();
        let window = state.window().clone();
        
        self.input.handle_window_event(&event);
        state.handle_event(&event);
        if window.id() == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    let mut gui_controller = self.gui_controller.as_mut().unwrap();

                    if self.exit_level {
                        self.level = None;
                        self.exit_level = false;
                    };
            
                    let mut debug_data = String::new();
                    self.client_engine.player().handle_input(&self.input, self.time.delta(), false);
                    let mesh_vec = if let Some(level) = &mut self.level {
                        let result = level.update(
                            &self.input,
                            &self.time,
                            &mut state,
                            &mut gui_controller,
                            &mut self.debug_block_id,
                            self.setting.render_radius,
                            &self.client_engine.player()
                        );
                        let player = self.client_engine.player();
                        debug_data += &format!("{:?}", player.camera().position_tuple());
                        state.update_camera(&player.camera().proj_view(state.size.width as f32, state.size.height as f32).into());
                        let (sun, sky) = level.sun.sun_sky();
                        state.set_sun_color(sun.into());
                        state.set_clear_color(sky.into());
                        // println!("Chunks: {}", unsafe {&*level.world.chunks.chunks.get()}.len());
                        if self.input.is_key(&Key::KeyE, KeypressState::AnyJustPress) {
                            gui_controller.set_cursor_lock(player.is_inventory);
                            state.set_ui_interaction(player.is_inventory);
                        }
                        result
                    } else {vec![]};
                    self.client_engine.tick();
                    
                    self.time.update();
                    state.update_time(self.time.current());
            
                    gui_controller.update_cursor_lock();
            
                    self.fps_queue.push_back(1.0/self.fps.elapsed().as_secs_f32());
                    debug_data += &(self.fps_queue.iter().sum::<f32>() / self.fps_queue.len() as f32).floor().to_string();
                    self.fps_queue.pop_front();
                    self.fps = Instant::now();
            
                    if self.input.is_key(&Key::F1, KeypressState::AnyJustPress) {
                        gui_controller.toggle_ui();
                        state.set_crosshair(gui_controller.is_ui());
                    }
                    
                    if self.input.is_key(&Key::F11, KeypressState::AnyJustPress) {
                        let window = state.window();
                        if window.fullscreen().is_some() {
                            window.set_fullscreen(None);
                        } else {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        }
                    }
            
                    if self.input.is_key(&Key::F6, KeypressState::AnyJustPress) {
                        self.level.as_ref().unwrap().world.chunks.chunk((0, 0).into()).unwrap().light_map().0
                            .iter().for_each(|l| println!("{:?}", l.get_normalized()));
                        // println!("{:?}", self.level.as_ref().unwrap().world.chunks.voxel_global(GlobalCoord::new(1, 1, 1)));
                        // println!("{:?}", self.level.as_ref().unwrap().world.chunks.voxel_global(GlobalCoord::new(33, 1, 33)));
                        // for chunk in unsafe {&*self.level.as_ref().unwrap().world.chunks.chunks.get()}.values() {
                        //     println!("{:?}", chunk.coord);
                        //     println!("{:?}", chunk.voxels().get(LocalCoord::new(0, 0, 0)));
                        // }
                    }
            
                    if self.input.is_key(&Key::Escape, KeypressState::AnyJustPress) {
                        gui_controller.toggle_menu();
                        gui_controller.set_cursor_lock(gui_controller.is_menu);
                        state.set_ui_interaction(gui_controller.is_menu);
                    }
            
                    let players_mesh: Vec<PlayerMesh> = self.client_engine.positions().into_iter()
                        .map(|pp| PlayerMesh::new(&state, pp)).collect();
                    match state.render(&mesh_vec, &players_mesh, |ctx| {
                        if let Some(l) = &self.level {
                            let mut player = unsafe {l.player.lock_unsafe()}.unwrap();
                            gui_controller
                                .draw_inventory(ctx, &mut player)
                                .draw_debug(ctx, &debug_data, &mut self.debug_block_id)
                                .draw_active_recieps(ctx, &mut player);
            
                            drop(player);
                            gui_controller.draw_in_game_menu(ctx, &mut self.exit_level);
                        } else {
                            gui_controller
                                .draw_main_screen(ctx, event_loop, &mut self.world_loader, &mut self.setting, &mut self.level, &self.indices);
                        }
            
                        gui_controller.draw_setting(ctx, &mut self.setting, &self.save.setting);
                    }) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.size)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(wgpu::SurfaceError::Timeout) => eprintln!("Surface timeout"),
                    }
                    self.input.update();
                    state.window().request_redraw();
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.state.as_ref().unwrap().window();
        window.request_redraw();
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.input.handle_device_event(event);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: crate::Timer) {
        match event {
            crate::Timer::Tick => {
                self.tick += 1;
            },
            crate::Timer::Second => {
                println!("{}", self.tick);
            },
            crate::Timer::Minute => {
                println!("Minute");
            }
        }
    }
}