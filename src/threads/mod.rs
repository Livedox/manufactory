use std::{thread::JoinHandle, sync::{Arc, mpsc::Sender, Mutex, Condvar, atomic::{Ordering, AtomicBool}, RwLock}};

use crate::{content::Content, graphic::render::RenderResult, player::player::Player, save_load::{WorldRegions, WorldSaver}, unsafe_mutex::UnsafeMutex, world::World, WORLD_EXIT};

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
    save_condvar: Arc<(Mutex<SaveState>, Condvar)>,
    exit: Arc<AtomicBool>,
}

impl Threads {
    pub fn new(
        content: Arc<Content>,
        world: Arc<World>,
        player: Arc<UnsafeMutex<Player>>,
        world_saver: Arc<WorldSaver>,
        sender: Sender<RenderResult>,
        save_condvar: Arc<(Mutex<SaveState>, Condvar)>,
    ) -> Self {
        let exit = Arc::new(AtomicBool::new(false));
        Self {
            save: save::spawn(world.clone(), player, world_saver.clone(), save_condvar.clone()),
            world_loader: world_loader::spawn(world.clone(), world_saver.regions.clone(), exit.clone()),
            voxel_data_updater: voxel_data_updater::spawn(world.clone(), exit.clone()),
            renderer: renderer::spawn(content, world, sender, exit.clone()),
            save_condvar,
            exit
        }
    }


    pub fn finalize(self) {
        self.exit.store(true, Ordering::Release);
        if self.voxel_data_updater.join().is_err() {
            eprintln!("Failed to finish voxel_data_updater thread!");
        }
        if self.world_loader.join().is_err() {
            eprintln!("Failed to finish world_loader thread!");
        }
        if self.renderer.join().is_err() {
            eprintln!("Failed to finish renderer thread!");
        }
        let (save_state, cvar) = &*self.save_condvar;
        *save_state.lock().unwrap() = SaveState::WorldExit;
        cvar.notify_one();
        if self.save.join().is_err() {
            eprintln!("Failed to finish save thread!");
        }
    }
}