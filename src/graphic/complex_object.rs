use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
/// 0: position, 1: uv
pub struct ComplexObjectVertex(pub [f32; 3], pub [f32; 2]);

impl ComplexObjectVertex {
    #[inline]
    pub fn x(&self, rotation_index: usize) -> f32 {
        if rotation_index > 1 {self.0[2]} else {self.0[0]}
    }

    #[inline] pub fn y(&self) -> f32 {self.0[1]}

    #[inline]
    pub fn z(&self, rotation_index: usize) -> f32 {
        if rotation_index > 1 {self.0[0]} else {self.0[2]}
    }

    #[inline] pub fn u(&self) -> f32 {self.1[0]}
    #[inline] pub fn v(&self) -> f32 {self.1[1]}
}

impl From<([f32; 3], [f32; 2])> for ComplexObjectVertex {
    fn from(value: ([f32; 3], [f32; 2])) -> Self {
        Self (value.0, value.1)
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ComplexObjectGroup(pub [ComplexObjectVertex; 4]);

impl ComplexObjectGroup {
    const VERTEX_ORDER: [[usize; 4]; 4] = [
        [0, 1, 2, 3],
        [2, 3, 0, 1],
        [3, 2, 1, 0],
        [1, 0, 3, 2]
    ];

    #[inline]
    pub fn get(&self, rotation_index: usize, index: usize) -> &ComplexObjectVertex {
        &self.0[Self::VERTEX_ORDER[rotation_index][index]]
    }

    #[inline]
    pub fn x(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].x(rotation_index)
    }

    #[inline]
    pub fn y(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].y()
    }

    #[inline]
    pub fn z(&self, rotation_index: usize, index: usize) -> f32 {
        self.0[Self::VERTEX_ORDER[rotation_index][index]].z(rotation_index)
    }


    pub fn sum_position(&self, x: f32, y: f32, z: f32, rotation_index: usize, index: usize) -> [f32; 3] {
        [self.x(rotation_index, index) + x,
         self.y(rotation_index, index) + y,
         self.z(rotation_index, index) + z]
    }

    #[inline] pub fn u(&self, index: usize) -> f32 {self.0[index].u()}
    #[inline] pub fn v(&self, index: usize) -> f32 {self.0[index].v()}
    #[inline] pub fn uv(&self, index: usize) -> [f32; 2] {[self.0[index].u(), self.0[index].v()]}
}

impl From<[ComplexObjectVertex; 4]> for ComplexObjectGroup {
    fn from(value: [ComplexObjectVertex; 4]) -> Self {
        Self(value)
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ComplexObjectSide {
    pub texture_layer: u32,
    pub vertex_groups: Vec<ComplexObjectGroup>,
}

impl ComplexObjectSide {
    #[inline]
    pub fn new(texture_layer: u32, vertex_groups: Vec<ComplexObjectGroup>) -> Self {
        Self { texture_layer, vertex_groups }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]

pub struct ComplexObject {
    pub block: Box<[[Option<ComplexObjectSide>; 6]]>,
    pub transport_belt: Box<[[Option<ComplexObjectSide>; 6]]>,
    pub model_names: Box<[String]>,
    pub animated_models_names: Box<[String]>,
}

impl Default for ComplexObject {
    fn default() -> Self {
        Self {
            block: Box::new([]),
            transport_belt: Box::new([]),
            model_names: Box::new([]),
            animated_models_names: Box::new([]),
        }
    }
}

pub fn load_complex_object(name: &str) -> ComplexObject {
    let data = std::fs::read(format!("./complex_objects/{}", name)).unwrap();
    serde_json::from_slice(&data).unwrap()
}