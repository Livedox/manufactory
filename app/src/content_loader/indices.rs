use std::{collections::HashMap, path::{Path, PathBuf}};

use graphics_engine::{constants::{BLOCK_MIPMAP_COUNT, BLOCK_TEXTURE_SIZE}, models::model::Model};
use image::{imageops::FilterType, DynamicImage};
use itertools::Itertools;
use resources::texture::{load_texture, ModelTexture};

#[derive(Debug, Clone)]
pub struct GamePath<T: AsRef<Path>> {
    pub path: T,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Indices {
    pub block: HashMap<String, u32>,
    pub models: HashMap<String, u32>,
    pub animated_models: HashMap<String, u32>,
}

pub fn load_blocks_textures(paths: &[GamePath<PathBuf>]) -> (HashMap::<String, u32>, Vec<Vec<u8>>, u32) {
    let (names, images): (Vec<String>, Vec<DynamicImage>) = paths.iter().flat_map(|path| {
            let prefix = path.prefix.as_ref().map(|s| s.as_str()).unwrap_or("");
            let files = walkdir::WalkDir::new(&path.path)
                .into_iter()
                .filter_map(|f| f.ok())
                .filter(|f| f.file_type().is_file());

            files.map(move |file| {
                let name = file.file_name().to_str().unwrap();
                let dot_index = name.rfind('.').unwrap();
                (format!("{}{}", &prefix, &name[..dot_index]), image::open(file.path())
                    .unwrap_or_else(|_| panic!("Failed to open image on path: {:?}", file.path())))
            })
        }).unzip();

    let data = (0..BLOCK_MIPMAP_COUNT).map(|mipmap| {
        let size = BLOCK_TEXTURE_SIZE / 2u32.pow(mipmap as u32);
        images.iter().flat_map(|image| {
            if mipmap == 0 {return image.to_rgba8().to_vec()};
            image.resize(size, size, FilterType::Triangle).to_rgba8().to_vec()
        }).collect_vec()
    }).collect_vec();

    let indices: HashMap<String, u32> = names.into_iter().enumerate()
        .map(|(i, n)| (n, i as u32)).collect();

    (indices, data, images.len() as u32)
}

pub fn load_animated_models(
    model_paths: &[impl AsRef<Path>],
    texture_paths: &[impl AsRef<Path>],
) -> (HashMap::<String, u32>, Vec<resources::animated_model::AnimatedModel>) {
    let load = |p: &Path, m: ModelTexture| {resources::animated_model::load_animated_model(p, m)};
    load_with_texture(model_paths, texture_paths, &load)
}

pub fn load_models(
    model_paths: &[impl AsRef<Path>],
    texture_paths: &[impl AsRef<Path>], 
) -> (HashMap::<String, u32>, Vec<resources::model::Model>) {
    let load = |p: &Path, m: ModelTexture| {resources::model::load_model(p, m)};
    let (indices, models) = load_with_texture(model_paths, texture_paths, &load);
    (indices, models.into_iter().map(|m| m.unwrap()).collect())
}

pub fn load_with_texture<T>(
    paths: &[impl AsRef<Path>],
    texture_paths: &[impl AsRef<Path>],
    load: &dyn for<'a> Fn(&'a Path, ModelTexture) -> T
) -> (HashMap::<String, u32>, Vec<T>) {
    let textures_files = texture_paths.iter().flat_map(|path| walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file()));

    let mut textures: HashMap<String, ModelTexture> = textures_files.map(|file| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        load_texture(file.path()).ok().map(|t| (name, t))
    }).flatten().collect();

    let files = paths.iter().flat_map(|path| walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate());

    let mut indices = HashMap::<String, u32>::new();
    let datas: Vec<T> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let data = load(file.path(), textures.remove(&name).unwrap());
        indices.insert(name, index as u32);
        data
    }).collect();

    (indices, datas)
}