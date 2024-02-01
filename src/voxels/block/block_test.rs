use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{direction::Direction, player::player::Player, recipes::{item::Item, storage::Storage}, world::{coords::Coords, global_coords::GlobalCoords, World}};

use super::{block_type::BlockType, functions::{on_break, Function}};

fn one() -> usize {1}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Faces {
    One(String),
    All(Vec<String>)
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BlockTypeFile {
    #[serde(rename = "complex_object")]
    ComplexObject { cp: String },
    #[serde(rename = "block")]
    Block { faces: Faces },
    #[serde(rename = "model")]
    Model { name: String },
    #[serde(rename = "animated_model")]
    AnimatedModel { name: String },
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockFile {
    pub id: String,
    pub block_type: BlockTypeFile,

    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub emission: [u8; 3],

    #[serde(default = "one")]
    pub width: usize,
    #[serde(default = "one")]
    pub height: usize,
    #[serde(default = "one")]
    pub depth: usize,

    #[serde(default)]
    pub is_light_passing: bool,
    #[serde(default)]
    pub is_additional_data: bool,
    #[serde(default)]
    pub is_glass: bool,
    #[serde(default)]
    pub is_ore: bool,
}


pub fn to_block(block_file: BlockFile, block_texture_id: &HashMap<String, u32>, id: u32) -> Block {
    let block_type = match &block_file.block_type {
        BlockTypeFile::Block { faces } => {
            let faces = match faces {
                Faces::One(texture) => {
                    let id = *block_texture_id.get(texture).unwrap();
                    [id, id, id, id, id, id]
                },
                Faces::All(textures) => {
                    [0, 1, 2, 3, 4, 5].map(|i| {
                        *block_texture_id.get(&textures[i%textures.len()]).unwrap()
                    })
                },
            };
            BlockType::Block { faces }
        }
        BlockTypeFile::ComplexObject { cp } => todo!(),
        BlockTypeFile::Model { name } => todo!(),
        BlockTypeFile::AnimatedModel { name } => todo!(),
        BlockTypeFile::None => BlockType::None,
    };

    Block {
        base: BlockBase {
            id,
            item_id: None,
            emission: block_file.emission,
            block_type,
            width: block_file.width,
            height: block_file.height,
            depth: block_file.depth,
            is_light_passing: block_file.is_light_passing,
            is_additional_data: block_file.is_additional_data,
            is_glass: block_file.is_glass,
            is_ore: block_file.is_ore
        },
        on_block_break: Box::new([&on_break]),
        on_block_set: Box::new([])
    }
}

pub fn test_serde_block() {
    let b = BlockFile {
        id: String::from("iron_ore"),
        block_type: BlockTypeFile::Block { faces: Faces::One(String::from("iron_ore")) },
        item_id: None,
        emission: [0, 0, 0],
        width: 1,
        height: 1,
        depth: 1,
        is_light_passing: false,
        is_additional_data: false,
        is_glass: false,
        is_ore: false,
    };

    std::fs::write("./block.json", serde_json::to_vec_pretty(&b).unwrap()).unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockBase {
    pub id: u32,
    pub item_id: Option<u32>,
    pub emission: [u8; 3],
    pub block_type: BlockType,

    pub width: usize,
    pub height: usize,
    pub depth: usize,
    
    pub is_light_passing: bool,
    pub is_additional_data: bool,
    pub is_glass: bool,
    pub is_ore: bool,
}

pub struct Block {
    pub base: BlockBase,

    // pub on_use: Box<[Function]>,
    pub on_block_break: Box<[Function]>,
    pub on_block_set: Box<[Function]>
}

impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base.fmt(f)
    }
}

impl Block {
    // #[inline]
    // pub fn on_use(&self, world: &World, player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
    //     if self.on_block_break.is_empty() {return false;}
    //     self.on_block_break.iter().all(|f| f(&self.base, world, player, xyz, dir))
    // }

    #[inline]
    pub fn on_block_break(&self, world: &World, player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
        if self.on_block_break.is_empty() {return false;}
        self.on_block_break.iter().all(|f| f(&self.base, world, player, xyz, dir))
    }

    #[inline]
    pub fn on_block_set(&self, world: &World, player: &mut Player, xyz: &GlobalCoords, dir: &Direction) -> bool {
        if self.on_block_set.is_empty() {return false;}
        self.on_block_set.iter().all(|f| f(&self.base, world, player, xyz, dir))
    }

    pub fn id(&self) -> u32 {self.base.id}
    pub fn emission(&self) -> &[u8; 3] {&self.base.emission}
    pub fn is_light_passing(&self) -> bool {self.base.is_light_passing}
    pub fn block_type(&self) -> &BlockType {&self.base.block_type}
    pub fn is_additional_data(&self) -> bool {self.base.is_additional_data}
    pub fn is_glass(&self) -> bool {self.base.is_glass}
    
    pub fn width(&self) -> usize {1}
    pub fn height(&self) -> usize {1}
    pub fn depth(&self) -> usize {1}
    pub fn min_point(&self) -> &Coords {&Coords(0.0, 0.0, 0.0)}
    pub fn max_point(&self) -> &Coords {&Coords(1.0, 1.0, 1.0)}
    pub fn is_multiblock(&self) -> bool {false}
    pub fn is_voxel_size(&self) -> bool {false}

    pub fn ore(&self) -> Option<Item> {
        if self.base.is_ore {
            if let Some(id) = self.base.item_id {
                return Some(Item::new(id, 1));
            }
        }
        None
    }
}