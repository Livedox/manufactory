use std::collections::HashMap;
use std::path::Path;


use serde::{Deserialize, Serialize};
use crate::Indices;
use crate::graphic::complex_object::{load_complex_object, ComplexObject};
use crate::{voxels::{block::{block_test::{to_block, Block, BlockBase, BlockFile}, block_type::BlockType, functions::{on_multiblock_break}}, live_voxels::{register, LiveVoxelRegistrator}}};

pub fn load_complex_objects(
    complex_objects_path: impl AsRef<Path>,
    tmp_indices: &Indices
) -> (HashMap::<String, u32>, Box<[ComplexObject]>) {
    let files = walkdir::WalkDir::new(complex_objects_path)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .enumerate();
    let mut indices = HashMap::<String, u32>::new();
    let complex_objects: Box<[ComplexObject]> = files.map(|(index, file)| {
        let file_name = file.file_name().to_str().unwrap();
        let dot_index = file_name.rfind('.').unwrap();
        let name = file_name[..dot_index].to_string();
        let model = load_complex_object(file.path(), tmp_indices);
        indices.insert(name, index as u32);
        model
    }).collect();

    (indices, complex_objects)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentIndices {
    pub blocks: Vec<String>,
}

#[derive(Debug)]
pub struct Content {
    pub block_indexes: HashMap<String, u32>,
    pub co_indices: HashMap<String, u32>,
    pub blocks: Vec<Block>,
    pub complex_objects: Box<[ComplexObject]>,

    pub live_voxel: LiveVoxelRegistrator,
}

impl Content {
    pub fn new(indices: &Indices, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().join("indices.json");
        let content_indices = if let Ok(bytes) = std::fs::read(&path) {
            Some(serde_json::from_slice::<ContentIndices>(&bytes).unwrap())
        } else {
            None
        };
        let mut block_indexes = HashMap::<String, u32>::new();
        let (co_indices, complex_objects) = load_complex_objects("./res/complex_objects", indices);
        let files = walkdir::WalkDir::new("./res/blocks/")
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file());

        let mut blocks = vec![
            Block {
                base: BlockBase {
                    id: 0,
                    item_id: None,
                    emission: [0, 0, 0],
                    block_type: BlockType::None,
                    width: 1,
                    height: 1,
                    depth: 1,
                    is_light_passing: true,
                    live_voxel: None,
                    is_glass: false,
                    is_ore: false
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 1,
                    item_id: None,
                    emission: [0, 0, 0],
                    block_type: BlockType::None,
                    width: 1,
                    height: 1,
                    depth: 1,
                    is_light_passing: true,
                    live_voxel: None,
                    is_glass: false,
                    is_ore: false
                },
                on_block_break: Box::new([&on_multiblock_break]),
                on_block_set: Box::new([]),
            },
        ];
        let blocks_init_len = blocks.len();
        if let Some(content) = content_indices {
            let mut all_count = 0;
            let mut id = blocks.len() as u32 + content.blocks.len() as u32;
            files.for_each(|file| {
                let file_name = file.file_name().to_str().unwrap();
                let dot_index = file_name.rfind('.').unwrap();
                let name = file_name[0..dot_index].to_string();
                let data = std::fs::read(file.path()).unwrap();
                let block_file: BlockFile = serde_json::from_slice(&data).unwrap();

                if let Some(position) = content.blocks.iter().position(|s| s == &name) {
                    all_count += 1;
                    block_indexes.insert(name, position as u32 + blocks_init_len as u32);
                    blocks.push(to_block(block_file, indices, &co_indices, position as u32 + blocks_init_len as u32));
                } else {
                    block_indexes.insert(name, id);
                    blocks.push(to_block(block_file, indices, &co_indices, id));
                    id += 1;
                };
            });
            if all_count < content.blocks.len() {
                panic!("Missing contents");
            }
        } else {
            let mut id = blocks.len() as u32;
            files.for_each(|file| {
                let name = file.file_name().to_str().unwrap();
                let dot_index = name.rfind('.').unwrap();
                block_indexes.insert(name[0..dot_index].to_string(), id);

                let data = std::fs::read(file.path()).unwrap();
                let block_file: BlockFile = serde_json::from_slice(&data).unwrap();

                blocks.push(to_block(block_file, indices, &co_indices, id));
                id += 1;
            });
        }
        
        let mut content_blocks = vec![String::new(); block_indexes.len()];
        block_indexes.iter().for_each(|(s, i)| {
            content_blocks[*i as usize - blocks_init_len] = s.clone();
        });
        println!("{:?} {:?}", content_blocks, block_indexes);
        let content_indices = ContentIndices {blocks: content_blocks};
        let data = serde_json::to_vec_pretty(&content_indices).unwrap();
        std::fs::write(path, data).unwrap();

        Self { blocks, block_indexes, live_voxel: register(), co_indices, complex_objects }
    }
}