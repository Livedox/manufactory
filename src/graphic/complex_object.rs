use std::path::Path;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{Indices};
#[derive(Deserialize, Serialize, Debug, Clone)]
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
#[derive(Deserialize, Serialize, Debug, Clone)]
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
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComplexObjectSide {
    pub texture_layer: u32,
    pub vertex_group: ComplexObjectGroup,
}

impl ComplexObjectSide {
    #[inline]
    pub fn new(texture_layer: u32, vertex_group: ComplexObjectGroup) -> Self {
        Self { texture_layer, vertex_group }
    }
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]

pub struct ComplexObject {
    pub block: [Vec<ComplexObjectSide>; 6],
    pub transport_belt: [Vec<ComplexObjectSide>; 6],
    pub models: Vec<u32>,
    pub animated_models: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexObjectSideFile {
    pub texture_layer: String,
    pub vertex_group: ComplexObjectGroup,
}

impl ComplexObjectSideFile {
    pub fn to_complex_object_side(self, indices: &Indices) -> ComplexObjectSide {
        ComplexObjectSide {
            texture_layer: *indices.block.get(&self.texture_layer).unwrap(),
            vertex_group: self.vertex_group,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexObjectFile {
    pub block: [Vec<ComplexObjectSideFile>; 6],
    pub transport_belt: [Vec<ComplexObjectSideFile>; 6],
    pub models: Vec<String>,
    pub animated_models: Vec<String>,
}

impl ComplexObjectFile {
    pub fn to_sides(array: [Vec<ComplexObjectSideFile>; 6], indices: &Indices) -> [Vec<ComplexObjectSide>; 6] {
        array.map(|sides| {
            sides.into_iter()
                .map(|side| side.to_complex_object_side(indices))
                .collect_vec()
        })
    }

    pub fn to_complex_object(self, indices: &Indices) -> ComplexObject {
        ComplexObject {
            block: Self::to_sides(self.block, indices),
            transport_belt: Self::to_sides(self.transport_belt, indices),
            models: self.models.into_iter()
                .map(|s| *indices.models.get(&s).unwrap()).collect_vec(),
            animated_models: self.animated_models.into_iter()
                .map(|s| *indices.animated_models.get(&s).unwrap()).collect_vec()
        }
    }
}

pub fn load_complex_object(path: impl AsRef<Path>, indices: &Indices) -> ComplexObject {
    let data = std::fs::read(path).unwrap();
    let complex_object_file: ComplexObjectFile = serde_json::from_slice(&data).unwrap();
    complex_object_file.to_complex_object(indices)
}