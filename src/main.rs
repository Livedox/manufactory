use std::{iter, time::{Instant, Duration, self}, cell::RefCell, sync::Arc, borrow::Borrow};

use input_event::KeypressState;
use light::light_solver::LightSolver;
use vertices::block_vertex::BlockVertex;
use voxels::{chunks::Chunks, chunk::CHUNK_SIZE, block::BLOCKS};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, Fullscreen}, dpi::PhysicalPosition,
};
use async_std::task::block_on;

use crate::{input_event::input_service::{Key, Mouse}, voxels::ray_cast};
use nalgebra_glm as glm;

mod texture;
mod state;
mod input_event;
mod my_time;
mod voxels;
mod graphic;
mod light;
mod vertices;
mod mipmaps;
mod meshes;
mod camera;
mod cursor_lock;

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut cursor_lock = cursor_lock::CursorLock::new();
    let mut camera = camera::camera_controller::CameraController::new(glm::vec3(0.0, 0.0, 0.0), 1.2);
    let mut meshes = meshes::Meshes::new();
    let mut input = input_event::input_service::InputService::new();
    let mut time = my_time::Time::new();


    let mut voxel_renderer = graphic::render::VoxelRenderer {};
    let chunks = Arc::new(RefCell::new(Chunks::new(2, 1, 2, 0, 0, 0)));
    loop { if !chunks.borrow_mut().load_visible() {break;} };
    println!("{:?}", chunks.borrow_mut().chunks.len());
    chunks.borrow_mut().set(0, 0, 0, 0);
    chunks.borrow_mut().set(10, 10, 10, 0);

    let mut solver_red = LightSolver::new(chunks.clone(), 0);
	let mut solver_green = LightSolver::new(chunks.clone(), 1);
	let mut solver_blue = LightSolver::new(chunks.clone(), 2);
	let _solver_sun = LightSolver::new(chunks.clone(), 3);
    let height = chunks.borrow_mut().height;
    let depth = chunks.borrow_mut().depth;
    let width = chunks.borrow_mut().width;
    for y in 0..height*CHUNK_SIZE as i32 {
        for z in 0..depth*CHUNK_SIZE as i32  {
            for x in 0..width*CHUNK_SIZE as i32 {
                let chunks = chunks.borrow_mut();
                let vox = chunks.voxel_global(x,y,z);
                let block = &BLOCKS[vox.unwrap().id as usize];
                if block.emission.iter().any(|i| *i > 0) {
                    drop(vox);
                    drop(chunks);
                    solver_red.add_with_emission(x,y,z,block.emission[0]);
                    solver_green.add_with_emission(x,y,z,block.emission[1]);
                    solver_blue.add_with_emission(x,y,z,block.emission[2]);
                }
            }
        }
    }

    solver_red.solve();
    solver_blue.solve();
    solver_green.solve();

    // let mut is_meshes_changes = false;
    // loop {
    //     if chunks.borrow_mut().build_meshes(&mut voxel_renderer).is_none() { break; }
    //     is_meshes_changes = true;
    // };

    


    // let mut block_buffer: Vec<BlockVertex> = vec![];
    // let mut index_buffer: Vec<u16> = vec![];
    // let mut offset = 0;
    // chunks.borrow_mut().meshes.iter().for_each(|mesh| {
        
    //     if let Some(mesh) = mesh {
    //         mesh.0.iter().for_each(|v| {
    //             block_buffer.push(*v);
    //         });
    //         mesh.1.iter().for_each(|u| {
    //             index_buffer.push(offset+u);
    //         });
    //         offset += block_buffer.len() as u16;
    //     }
    // });


    let mut state = state::State::new(window, &camera.proj_view(400.0 as f32, 400.0 as f32).into());
    loop {
        let index = chunks.borrow_mut().get_nearest_chunk_index();
        if let Some(index) = index {
            meshes.render(state.device(), &mut voxel_renderer, &mut chunks.borrow_mut(), index);
        } else {
            break;
        }
    }
    

    let mut fps = 0;
    event_loop.run(move |event, _, control_flow| {
        input.process_events(&mut time, &event);
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
                state.update(&camera.proj_view(state.size.width as f32, state.size.height as f32).into());
                cursor_lock.update(&state.window());
                if input.is_keys(&[(Key::E, KeypressState::AnyJustPress)]) {
                    cursor_lock.set_cursor_lock(&state.window(), !cursor_lock.is_cursor()) 
                }
                
                time.update();
                camera.update(&input, state.size.height as f32, &time, cursor_lock.is_cursor());

                fps += 1;
                if time.is_more_then_1s() {
                    println!("fps: {}", fps);
                    fps = 0;
                }


                {
                    let mut chunks = chunks.borrow_mut();
                    let result = ray_cast::ray_cast(&chunks, camera.position(), camera.front(), 10.0);
                    match result {
                        Some(result) => {
                            let (x, y, z, voxel, norm) = ((result.0) as i32, (result.1) as i32, (result.2) as i32, result.3, result.4);
                            let (v_x, v_y, v_z) = (result.0, result.1, result.2);
                            
                            let _id = if voxel.is_some() { voxel.unwrap().id } else { 0 };
                            if input.is_mouse(&[(Mouse::Left, KeypressState::AnyPress)]) {
                                chunks.set(x, y, z, 0);
                                drop(chunks);
                                solver_red.remove(x,y,z);
                                solver_green.remove(x,y,z);
                                solver_blue.remove(x,y,z);
                                solver_red.solve();
                                solver_green.solve();
                                solver_blue.solve();
                                
                                solver_red.add(x,y+1,z); solver_green.add(x,y+1,z); solver_blue.add(x,y+1,z);
                                solver_red.add(x,y-1,z); solver_green.add(x,y-1,z); solver_blue.add(x,y-1,z);
                                solver_red.add(x+1,y,z); solver_green.add(x+1,y,z); solver_blue.add(x+1,y,z);
                                solver_red.add(x-1,y,z); solver_green.add(x-1,y,z); solver_blue.add(x-1,y,z);
                                solver_red.add(x,y,z+1); solver_green.add(x,y,z+1); solver_blue.add(x,y,z+1);
                                solver_red.add(x,y,z-1); solver_green.add(x,y,z-1); solver_blue.add(x,y,z-1);
    
                                solver_red.solve();
                                solver_green.solve();
                                solver_blue.solve();
                            } else if input.is_mouse(&[(Mouse::Right, KeypressState::AnyJustPress)]) {
                                let (t_x, t_y, t_z) = (x+norm.x as i32, y+norm.y as i32, z+norm.z as i32);
                                chunks.set(x+norm.x as i32, y+norm.y as i32, z+norm.z as i32, 3.try_into().unwrap());
                                drop(chunks);
                                solver_red.remove(t_x,t_y,t_z);
                                solver_green.remove(t_x,t_y,t_z);
                                solver_blue.remove(t_x,t_y,t_z);
                                let block = &BLOCKS[3 as usize];
                                if block.emission.iter().any(|i| *i > 0) {
                                    solver_red.add_with_emission(t_x,t_y,t_z,block.emission[0]);
                                    solver_green.add_with_emission(t_x,t_y,t_z,block.emission[1]);
                                    solver_blue.add_with_emission(t_x,t_y,t_z,block.emission[2]);
                                    
                                }
                                solver_red.solve();
                                solver_green.solve();
                                solver_blue.solve();
                            }
                        }
                        _ => {},
                    }
                }
                loop {
                    let index = chunks.borrow_mut().get_nearest_chunk_index();
                    if let Some(index) = index {
                        meshes.render(state.device(), &mut voxel_renderer, &mut chunks.borrow_mut(), index);
                    } else {
                        break;
                    }
                }
                

                match state.render(&meshes) {
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
