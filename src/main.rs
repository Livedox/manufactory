extern crate app;
use std::{collections::HashMap, ffi::OsStr};
use app::{content_loader::ContentLoader, voxels::live_voxels::{LiveVoxelRegistrator, LIVE_VOXEL_REGISTER}, Registrator};
use libloading::Library;

const LIB_FORMAT: &'static str = if cfg!(target_os = "windows") {
    "dll"
} else if cfg!(target_os = "linux") {
    "so"
} else if cfg!(target_os = "macos") {
    "dylib"
} else {
    "module"
};


pub fn main() {
<<<<<<< HEAD
=======
    println!("{}", cfg!(target_endian = "little"));
>>>>>>> arc_swap-test
    // let content_loader = ContentLoader::new("./res/content/");
    // let mut registrator = Registrator {
    //     c: HashMap::new(),
    //     from_bytes: HashMap::new(),
    // };
    // let libs: Vec<Library> = content_loader.details().values().filter(|d| d.active())
    //     .map(|d| {
    //         let path = d.path().join(&format!("mod.{}", LIB_FORMAT));
    //         load_library(path, &mut registrator).ok()
    //     }).flatten().collect();

<<<<<<< HEAD
    // // println!("{:?} {}", content_loader.load_indices("./res/game/"), libs.len());
=======
    // println!("{:?} {}", content_loader.load_indices("./res/game/"), libs.len());
>>>>>>> arc_swap-test

    // unsafe {
    //     LIVE_VOXEL_REGISTER = Some(
    //         LiveVoxelRegistrator {
    //             deserialize: registrator.from_bytes,
    //             new: registrator.c
    //         }
    //     )
    // }
    app::run();
    println!("Exit!");

    // libs.into_iter().for_each(|lib| lib.close().unwrap());
}

fn load_library(path: impl AsRef<OsStr>, registrator: &mut Registrator) -> Result<Library, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new(path)?;
        let init: libloading::Symbol<unsafe extern fn (registrator: &mut Registrator) -> ()> = 
            lib.get(b"init")?;
        init(registrator);
        Ok(lib)
    }
}