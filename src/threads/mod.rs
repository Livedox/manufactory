use std::{thread::JoinHandle, sync::{Arc, mpsc::Sender, Mutex, Condvar, atomic::Ordering}};

use crate::{unsafe_mutex::UnsafeMutex, world::World, save_load::WorldRegions, graphic::render::RenderResult, WORLD_EXIT};

use self::save::SaveState;

pub mod renderer;
pub mod world_loader;
pub mod voxel_data_updater;
pub mod save;


pub struct Threads {
    save: JoinHandle<()>,
    world_loader: JoinHandle<()>,
    voxel_data_updater: JoinHandle<()>,
    renderer: JoinHandle<()>,
}

impl Threads {
    pub fn new(
        world: Arc<UnsafeMutex<World>>,
        world_regions: Arc<UnsafeMutex<WorldRegions>>,
        sender: Sender<RenderResult>,
        save_condvar: Arc<(Mutex<SaveState>, Condvar)>,
    ) -> Self {
        Self {
            save: save::spawn(world.clone(), world_regions.clone(), save_condvar),
            world_loader: world_loader::spawn(world.clone(), world_regions),
            voxel_data_updater: voxel_data_updater::spawn(world.clone()),
            renderer: renderer::spawn(world, sender),
        }
    }


    pub fn finalize(self, save_condvar: Arc<(Mutex<SaveState>, Condvar)>) {
        WORLD_EXIT.store(true, Ordering::Release);
        if self.voxel_data_updater.join().is_err() {
            eprintln!("Failed to finish voxel_data_updater thread!");
        }
        if self.world_loader.join().is_err() {
            eprintln!("Failed to finish world_loader thread!");
        }
        if self.renderer.join().is_err() {
            eprintln!("Failed to finish renderer thread!");
        }
        let (save_state, cvar) = &*save_condvar;
        *save_state.lock().unwrap() = SaveState::WorldExit;
        cvar.notify_one();
        if self.save.join().is_err() {
            eprintln!("Failed to finish save thread!");
        }
    }
}