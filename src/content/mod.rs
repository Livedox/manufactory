use std::collections::HashMap;

use itertools::Itertools;

use crate::voxels::block::{block_test::{to_block, Block, BlockBase, BlockFile}, block_type::BlockType, functions::{on_break, player_add_item}};


#[derive(Debug)]
pub struct Content {
    pub blocks: Vec<Block>
}

impl Content {
    pub fn new(block_texture_id: &HashMap<String, u32>) -> Self {
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
                    is_additional_data: false,
                    is_glass: false,
                    is_ore: false
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            }
        ];
        let mut id: u32 = 1;
        files.for_each(|(index, file)| {
            let name = file.file_name().to_str().unwrap();
            let dot_index = name.rfind(".").unwrap();
            
            let data = std::fs::read(file.path()).unwrap();
            let block_file: BlockFile = serde_json::from_slice(&data).unwrap();

            blocks.push(to_block(block_file, block_texture_id, id));
            id += 1;
        });

        println!("{:?}", blocks);

        Self { blocks }
    }
}
    //     println!("{:?}", blocks);
    //     Self {
    //         blocks: vec![
    //             Block {
    //                 base: BlockBase {
    //                     id: 0,
    //                     item_id: Some(0),
    //                     block_type: BlockType::None,
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 1,
    //                     item_id: None,
    //                     block_type: BlockType::None,
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 2,
    //                     item_id: None,
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 3,
    //                     item_id: None,
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 4,
    //                     item_id: None,
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 5,
    //                     item_id: None,
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([&on_break, &player_add_item]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 6,
    //                     item_id: None,
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 7,
    //                     item_id: None,
    //                     block_type: BlockType::Block {
    //                         faces: [
    //                             *block_texture_id.get("rock").unwrap(),
    //                             *block_texture_id.get("rock").unwrap(),
    //                             *block_texture_id.get("rock").unwrap(),
    //                             *block_texture_id.get("rock").unwrap(),
    //                             *block_texture_id.get("rock").unwrap(),
    //                             *block_texture_id.get("rock").unwrap()
    //                         ]
    //                     },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([&on_break]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 8,
    //                     item_id: None,
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 9,
    //                     item_id: None,
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //             Block {
    //                 base: BlockBase {
    //                     id: 10,
    //                     item_id: Some(0),
    //                     block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
    //                     depth: 1,
    //                     height: 1,
    //                     width: 1,
    //                     emission: [0, 0, 0],
    //                     is_additional_data: false,
    //                     is_light_passing: true,
    //                     is_glass: false,
    //                     is_ore: false,
    //                 },
    //                 on_block_break: Box::new([]),
    //                 on_block_set: Box::new([]),
    //             },
    //         ]
    //     }
    // }
// }