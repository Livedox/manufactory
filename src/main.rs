extern crate app;
use std::collections::HashMap;

use app::{coords::global_coord::GlobalCoord, direction::{self, Direction}, voxels::live_voxels::{DesiarializeLiveVoxel, LiveVoxelBehavior, LiveVoxelCreation, LiveVoxelRegistrator, NewLiveVoxel, LIVE_VOXEL_REGISTER}, Registrator};
use libloading::Library;


pub fn main() {
    let mut registrator = Registrator {
        c: HashMap::new(),
        from_bytes: HashMap::new(),
    };
    let lib = load_library(&mut registrator).unwrap();
    unsafe {
        LIVE_VOXEL_REGISTER = Some(
            LiveVoxelRegistrator {
                deserialize: registrator.from_bytes,
                new: registrator.c
            }
        )
    }
    app::run();
    println!("Exit!");
    lib.close().unwrap();
}

fn load_library(registrator: &mut Registrator) -> Result<Library, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("./a/testmod.dll")?;
        let init: libloading::Symbol<unsafe extern fn (registrator: &mut Registrator) -> ()> = 
            lib.get(b"init")?;
        init(registrator);
        Ok(lib)
    }
}