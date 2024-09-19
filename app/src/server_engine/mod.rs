pub struct World {

}

pub struct ServerEngine {
    world: World,
}


impl ServerEngine {
    pub fn start() {
        loop {
            let mut command = String::new();
            std::io::stdin().read_line(&mut command).unwrap();
            println!("{command}");
        }
    }
}