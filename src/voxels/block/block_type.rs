use crate::graphic::complex_object::ComplexObject;

pub enum BlockType {
    ComplexObject {cp: ComplexObject},
    Block {faces: [u32; 6]}, // -x x -y y -z z
    Model {name: String},
    AnimatedModel {name: String},
    None,
}