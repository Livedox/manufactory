extern crate app;
use app::coords::global_coord::GlobalCoord;


pub fn main() {
    let coord = call_dynamic().unwrap();
    println!("coord {:?}", coord);
    app::run()
}

fn call_dynamic() -> Result<GlobalCoord, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("./mod.dll")?;
        let func: libloading::Symbol<unsafe extern fn() -> GlobalCoord> = lib.get(b"super_ultra_test")?;
        Ok(func())
    }
}