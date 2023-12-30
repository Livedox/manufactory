use std::{collections::HashMap, time::Instant};

use itertools::iproduct;

use crate::{voxels::{chunk::CHUNK_SIZE, chunks::Chunks, block::{blocks::BLOCKS, block_type::BlockType, light_permeability::LightPermeability}}, engine::vertices::block_vertex::BlockVertex, world::{World, chunk_coords::ChunkCoords}, engine::pipeline::IS_LINE, graphic::render::block_managers::BlockManagers};
use crate::light::light_map::Light;
use self::{model::{Models, ModelRenderResult, render_model}, animated_model::{AnimatedModels, AnimatedModelRenderResult, render_animated_model}, complex_object::render_complex_object, block::{BlockFaceLight, BlockFace, render_block}};

pub mod block_managers;
pub mod model;
pub mod animated_model;
pub mod complex_object;
mod block;

const IS_GREEDY_MESHING: bool = true;

#[derive(Debug, Clone)]
pub struct Buffer {
    pub buffer: Vec<BlockVertex>,
    pub index_buffer: Vec<u16>,
}

impl Buffer {
    pub fn new() -> Self { Self { buffer: vec![], index_buffer: vec![] }}

    pub fn push_line(&mut self, vertices: &[BlockVertex; 4], indices: &[usize]) {
        let mut index_vertex: [Option<usize>; 4] = [None, None, None, None];
        indices.iter().for_each(|i| {
            let current_index = self.buffer.len();
            index_vertex[*i] = Some(current_index);
            self.buffer.push(vertices[*i]);
            self.index_buffer.push(current_index as u16);
            if *i != 0 && *i < indices.len() - 1 {
                self.buffer.push(vertices[*i]);
                self.index_buffer.push(current_index as u16);
            }
        });
    }

    pub fn manage_vertices(&mut self, vertices: &[BlockVertex; 4], indices: &[usize]) {
        if !IS_LINE {
            self.push_triangles(vertices, indices);
        } else {
            self.push_line(vertices, indices);
        }
    }

    pub fn push_triangles(&mut self, vertices: &[BlockVertex], indices: &[usize]) {
        let mut index_vertex: [Option<usize>; 4] = [None, None, None, None];
        indices.iter().for_each(|i| {
            if let Some(index_vertex) = index_vertex[*i] {
                self.index_buffer.push(index_vertex as u16);
                return;
            }
            let current_index = self.buffer.len();
            index_vertex[*i] = Some(current_index);
            self.buffer.push(vertices[*i]);
            self.index_buffer.push(current_index as u16);
        });
    }
}

#[derive(Debug)]
pub struct RenderResult {
    pub chunk_index: usize,
    pub xyz: ChunkCoords,
    pub block_vertices: Vec<BlockVertex>,
    pub block_indices: Vec<u16>,
    pub belt_vertices: Vec<BlockVertex>,
    pub belt_indices: Vec<u16>,

    pub models: Models,
    pub animated_models: AnimatedModels,
}

pub fn render(chunk_index: usize, world: &World) -> Option<RenderResult> {
    let chunks = &world.chunks;
    let Some(Some(chunk)) = chunks.chunks.get(chunk_index).map(|c| c.as_ref()) else {return None};
    
    let mut models = Models::new();
    let mut animated_models = AnimatedModels::new();
    
    let mut block_manager = BlockManagers::new(IS_GREEDY_MESHING);
    
    let mut buffer = Buffer::new();
    let mut belt_buffer = Buffer::new();

    for (ly, lz, lx) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
        let id = unsafe {chunk.get_unchecked_voxel((lx, ly, lz).into()).id};
        if id == 0 { continue };
        let block = &BLOCKS()[id as usize];
        match block.block_type() {
            BlockType::Block {faces} => {
                render_block(&mut block_manager, chunks, chunk, block.as_ref(), faces, (lx, ly, lz));
            },
            BlockType::None => {},
            BlockType::Model {name} => {
                render_model(&mut models, chunk, name, lx, ly, lz);
            },
            BlockType::AnimatedModel {name} => {
                render_animated_model(&mut animated_models, chunk, name, lx, ly, lz);
            },
            BlockType::ComplexObject {cp} => {
                render_complex_object(cp, &mut buffer, &mut belt_buffer, chunk, lx, ly, lz);
            },
        };
    }
    let global = chunk.xyz.to_global((0u8, 0, 0).into()).into();
    block_manager.manage_vertices(&mut buffer, global);
    Some(RenderResult {
        chunk_index,
        xyz: chunk.xyz,
        block_vertices: buffer.buffer,
        block_indices: buffer.index_buffer,
        models,
        animated_models,
        belt_vertices: belt_buffer.buffer,
        belt_indices: belt_buffer.index_buffer,
    })
}