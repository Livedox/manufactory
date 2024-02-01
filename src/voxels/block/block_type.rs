use serde::{Deserialize, Serialize};

use crate::graphic::complex_object::ComplexObject;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum BlockType {
    ComplexObject {cp: ComplexObject},
    Block {faces: [u32; 6]}, // -x x -y y -z z
    Model {name: String},
    AnimatedModel {name: String},
    None,
}