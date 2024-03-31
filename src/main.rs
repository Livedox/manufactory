extern crate app;
use std::collections::HashMap;

use app::{coords::global_coord::GlobalCoord, direction::{self, Direction}, voxels::live_voxels::{DesiarializeLiveVoxel, LiveVoxelBehavior, LiveVoxelCreation, LiveVoxelRegistrator, NewLiveVoxel, LIVE_VOXEL_REGISTER}, Registrator};


pub fn main() {
    let mut registrator = Registrator {
        c: HashMap::new(),
        from_bytes: HashMap::new(),
    };
    unsafe {
        let lib = libloading::Library::new("./mod.dll").unwrap();
        let init: libloading::Symbol<unsafe extern fn (registrator: &mut Registrator) -> ()> = 
            lib.get(b"init").unwrap();
        init(&mut registrator);
        LIVE_VOXEL_REGISTER = Some(LiveVoxelRegistrator {
            new: registrator.c.into_iter().map(|(key, val)| {
                let a: NewLiveVoxel = Box::leak(val);
                (key, a)
            }).collect(),
            deserialize: registrator.from_bytes.into_iter().map(|(key, val)| {
                let a: DesiarializeLiveVoxel = Box::leak(val);
                (key, a)
            }).collect()
        });
        app::run()
    };    
}

fn call_dynamic(registrator: &mut Registrator) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("./mod.dll")?;
        let init: libloading::Symbol<unsafe extern fn (registrator: &mut Registrator) -> ()> = 
            lib.get(b"init")?;
        init(registrator)
    };
    Ok(())
}