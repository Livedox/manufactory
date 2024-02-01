use std::sync::OnceLock;
use super::block_test::{self, Block, BlockBase};
use crate::{voxels::block::{block_test::{}, functions::{on_break, player_add_item, FUNCTIONS}}};

use super::{interaction::BlockInteraction, block_ore::BlockOre, multiblock::MultiBlock, block_type::BlockType, block_builder::BlockBuilder, block_belt::BlockBelt};


static BLOCKS_CONTAINER: OnceLock<Vec<Block>> = OnceLock::new();

#[allow(non_snake_case)]
pub fn BLOCKS() -> &'static Vec<Block> {
    BLOCKS_CONTAINER.get_or_init(|| {
        let v = vec![
            Block {
                base: BlockBase {
                    id: 0,
                    item_id: Some(0),
                    block_type: BlockType::None,
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 1,
                    item_id: None,
                    block_type: BlockType::None,
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 2,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 3,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 4,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 5,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([&on_break, &player_add_item]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 6,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 7,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 8,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 9,
                    item_id: None,
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
            Block {
                base: BlockBase {
                    id: 10,
                    item_id: Some(0),
                    block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
                    depth: 1,
                    height: 1,
                    width: 1,
                    emission: [0, 0, 0],
                    is_additional_data: false,
                    is_light_passing: true,
                    is_glass: false,
                    is_ore: false,
                },
                on_block_break: Box::new([]),
                on_block_set: Box::new([]),
            },
        ];

        v
    })
}
// static BLOCKS_CONTAINER: OnceLock<Vec<Box<(dyn BlockInteraction + Send + Sync)>>> = OnceLock::new();

// #[allow(non_snake_case)]
// pub fn BLOCKS() -> &'static Vec<Box<(dyn BlockInteraction + Send + Sync)>> {
//     BLOCKS_CONTAINER.get_or_init(|| {
//         let blocks = vec![
//             //Air
//             BlockBuilder::new(0).is_light_passing(true).build(),
//             //Special Block
//             Box::new(MultiBlock {
//                 id: 1,
//                 emission: [0, 0, 0],
//                 is_light_passing: true,
//                 block_type: BlockType::None,
//                 is_additional_data: true,
//                 width: 1,
//                 height: 1,
//                 depth: 1
//             }),

//             BlockBuilder::new(2).faces(&[1]).build(),
//             BlockBuilder::new(3).faces(&[2]).build(),
//             BlockBuilder::new(4).faces(&[3]).emission([15, 15, 15]).build(),
//             Box::new(BlockOre{
//                 item_id: 0,
//                 id: 5,
//                 emission: [0, 0, 0],
//                 is_light_passing: false,
//                 block_type: BlockType::Block { faces: [4, 4, 4, 4, 4, 4] },
//                 is_additional_data: false,
//             }),
//             BlockBuilder::new(6).faces(&[6,6,6,5,6,6]).is_light_passing(true).build(),
//             Box::new(BlockOre{
//                 item_id: 3,
//                 id: 7,
//                 emission: [0, 0, 0],
//                 is_light_passing: false,
//                 block_type: BlockType::Block { faces: [9, 9, 9, 9, 9, 9] },
//                 is_additional_data: false,
//             }),
//             BlockBuilder::new(8).faces(&[7]).build(),
//             BlockBuilder::new(9).is_light_passing(true).animated_model_name(String::from("manipulator")).set_additional_data_true().build(),
//             BlockBuilder::new(10).is_light_passing(true).model_name(String::from("monkey")).emission([15, 10, 1]).build(),
//             BlockBuilder::new(11).is_light_passing(true).model_name(String::from("astronaut")).build(),
//             BlockBuilder::new(12).is_light_passing(true).animated_model_name(String::from("cowboy")).set_additional_data_true().build(),
//             BlockBuilder::new(13).faces(&[8]).set_additional_data_true().build(),
//             BlockBuilder::new(14).is_light_passing(true).model_name(String::from("furnace")).set_additional_data_true().build(),
//             Box::new(MultiBlock {
//                 id: 15,
//                 emission: [0, 0, 0],
//                 is_light_passing: true,
//                 block_type: BlockType::Model { name: String::from("drill") },
//                 is_additional_data: true,
//                 width: 2,
//                 height: 1,
//                 depth: 2
//             }),
//             Box::new(MultiBlock {
//                 id: 16,
//                 emission: [0, 0, 0],
//                 is_light_passing: true,
//                 block_type: BlockType::Model { name: String::from("assembler") },
//                 is_additional_data: true,
//                 width: 2,
//                 height: 2,
//                 depth: 2
//             }),
//             Box::new(BlockBelt {
//                 id: 17,
//                 emission: [0, 0, 0],
//                 is_light_passing: true,
//                 block_type: BlockType::ComplexObject { cp: new_transport_belt() },
//                 is_additional_data: true,
//             }),
//             BlockBuilder::new(18).faces(&[10]).is_light_passing(true).set_glass(true).build(),
//             BlockBuilder::new(19).faces(&[11]).is_light_passing(true).set_glass(true).build(),
//             BlockBuilder::new(20).faces(&[12]).is_light_passing(true).set_glass(true).build(),
//             BlockBuilder::new(21).faces(&[13]).is_light_passing(true).set_glass(true).build(),
//             BlockBuilder::new(22).faces(&[14]).is_light_passing(true).set_glass(true).build(),
//         ];

//         blocks
//     })
// }