use std::{thread::{self, JoinHandle}, sync::{Arc, Mutex, Condvar}, time::Duration};

use crate::{world::World, unsafe_mutex::UnsafeMutex, save_load::{WorldSaver}, player::player::Player};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SaveState {
    Unsaved,
    Saved,
    WorldExit,
}

pub fn spawn(
    world: Arc<World>,
    player: Arc<UnsafeMutex<Player>>,
    world_saver: Arc<WorldSaver>,
    save_condvar: Arc<(Mutex<SaveState>, Condvar)>
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let (lock, cvar) = &*save_condvar;
            let (mut save_state, _) = cvar.wait_timeout(lock.lock().unwrap(), Duration::new(60, 0)).unwrap();

            let mut world_regions = unsafe {world_saver.regions.lock_unsafe()}.unwrap();

            let mut chunks_awaiting_deletion = world.chunks.chunks_awaiting_deletion.lock().unwrap();
            chunks_awaiting_deletion.iter().for_each(|chunk| {
                world_regions.save_chunk(chunk);
            });
            chunks_awaiting_deletion.clear();
            drop(chunks_awaiting_deletion);

            unsafe {&*world.chunks.chunks.get()}.iter().for_each(|chunk| {
                let Some(chunk) = chunk.load_full() else {return};
                if !chunk.unsaved() {return};
                world_regions.save_chunk(&chunk);
                chunk.save(false);
            });
            world_regions.save_all_regions();

            let player = unsafe {player.lock_unsafe()}.unwrap();
            let player_saver = unsafe {world_saver.player.lock_unsafe()}.unwrap();
            player_saver.save_player(&player);

            if *save_state == SaveState::WorldExit {break};
            *save_state = SaveState::Saved;
        }
    })
}