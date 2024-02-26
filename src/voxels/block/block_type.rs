use serde::{Deserialize, Serialize};



#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum BlockType {
    ComplexObject {id: u32},
    Block {faces: [u32; 6]}, // -x x -y y -z z
    Model {id: u32},
    AnimatedModel {id: u32},
    None,
}