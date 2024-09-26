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
            println!("cdscsd");
            if let Some(result) = render(chunk.coord, &world.chunks, &content) {
                println!("cdsdc");
                sender.send(result).unwrap();
                println!("cdsdc");
            }
        } else {
            std::thread::sleep(Duration::from_millis(16));
        }
    }}).unwrap()
}