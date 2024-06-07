use std::{error::Error, fmt::Display, path::Path};

use image::ImageError;
use russimp::{scene::{PostProcess, Scene}, RussimpError};

use crate::texture::{load_texture, ModelTexture};

#[derive(Debug)]
pub enum ModelLoadingError {
    ImageError(ImageError),
    RussimpError(RussimpError),
}

impl Display for ModelLoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelLoadingError::ImageError(err) => err.fmt(f),
            ModelLoadingError::RussimpError(err) => err.fmt(f),
        }
    }
}

impl Error for ModelLoadingError {}

impl From<ImageError> for ModelLoadingError {
    fn from(value: ImageError) -> Self {
        Self::ImageError(value)
    }
}

impl From<RussimpError> for ModelLoadingError {
    fn from(value: RussimpError) -> Self {
        Self::RussimpError(value)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}
pub struct Model {
    pub vertices: Vec<ModelVertex>,
    pub texture: ModelTexture
}

pub fn load_model(
    src_model: impl AsRef<Path>,
    texture: ModelTexture,
) -> Result<Model, ModelLoadingError> {
    let scene = Scene::from_file(src_model.as_ref().to_str().unwrap(),
        vec![PostProcess::FlipUVs, PostProcess::MakeLeftHanded])?;

    let mut model_vertices: Vec<ModelVertex> = vec![];
    let mesh = &scene.meshes[0];

    mesh.vertices.iter().for_each(|vertex| {
        model_vertices.push(ModelVertex {
            position: [vertex.x + 0.5, vertex.y, vertex.z + 0.5],
            uv: [0.0, 0.0]});
    });

    mesh.texture_coords.iter().for_each(|coords| {
        if let Some(coords) = coords.as_ref() {
            coords.iter().enumerate().for_each(|(index, coords)| {
                model_vertices[index].uv = [coords.x, coords.y]})}
    });

    Ok(Model {
        vertices: model_vertices,
        texture,
    })
}