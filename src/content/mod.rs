use std::collections::HashMap;


use crate::graphic::complex_object::ComplexObject;
use crate::{engine::state::{load_complex_objects, Indices}, voxels::{block::{block_test::{to_block, Block, BlockBase, BlockFile}, block_type::BlockType, functions::{on_multiblock_break}}, live_voxels::{register, LiveVoxelRegistrator}}};

#[derive(Debug)]
pub struct Content {
    pub block_indexes: HashMap<String, u32>,
    pub co_indices: HashMap<String, u32>,
    pub blocks: Vec<Block>,
    pub complex_objects: Box<[ComplexObject]>,

    pub live_voxel: LiveVoxelRegistrator,
}

impl Content {
    pub fn new(indices: &Indices) -> Self {
        let mut block_indexes = HashMap::<String, u32>::new();
        let (co_indices, complex_objects) = load_complex_objects("./res/complex_objects", indices);
        let files = walkdir::WalkDir::new("./res/blocks/")
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file())
            .enumerate();

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
        let mut id = blocks.len() as u32;
        files.for_each(|(_, file)| {
            let name = file.file_name().to_str().unwrap();
            let dot_index = name.rfind('.').unwrap();
            block_indexes.insert(name[0..dot_index].to_string(), id);

            let data = std::fs::read(file.path()).unwrap();
            let block_file: BlockFile = serde_json::from_slice(&data).unwrap();

            blocks.push(to_block(block_file, indices, &co_indices, id));
            id += 1;
        });

        println!("{:?}", block_indexes);

        Self { blocks, block_indexes, live_voxel: register(), co_indices, complex_objects }
    }
}