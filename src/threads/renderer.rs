use std::{sync::{Arc, mpsc::Sender, atomic::{Ordering, AtomicBool}}, time::{Duration}, thread::{JoinHandle}};

use crate::{content::Content, graphic::render::{RenderResult, render}, world::{World}};

pub fn spawn(
    content: Arc<Content>,
    world: Arc<World>,
    sender: Sender<RenderResult>,
    exit: Arc<AtomicBool>
) -> JoinHandle<()> {
    let thread = std::thread::Builder::new().name("renderer".to_owned()).stack_size(32 * 1024 * 1024);
    thread.spawn(move || {loop {
        if exit.load(Ordering::Relaxed) {break};

        let ox = world.chunks.ox();
        let oz = world.chunks.oz();
        let width = world.chunks.width;
        let depth = world.chunks.depth;
        
        if let Some(chunk) = world.chunks.find_unrendered() {
            chunk.modify(false);
            if let Some(result) = render(chunk.xyz.nindex(width, depth, ox, oz), &world.chunks, &content) {
                let _ = sender.send(result);
            }
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
    }}).unwrap()
}