

use itertools::iproduct;
use graphics_engine::{constants::IS_LINE_TOPOLOGY, mesh::MeshInput, vertices::block_vertex::BlockVertex};
use crate::{content::Content, graphic::render::block_managers::BlockManagers, voxels::{block::block_type::BlockType, new_chunk::{LocalCoord, CHUNK_SIZE}, new_chunks::{ChunkCoord, Chunks, WORLD_BLOCK_HEIGHT, WORLD_HEIGHT}}};

use self::{model::{Models, render_model}, animated_model::{AnimatedModels, render_animated_model}, complex_object::render_complex_object, block::{render_block}};

pub mod block_managers;
pub mod model;
pub mod animated_model;
pub mod complex_object;
mod block;

const IS_GREEDY_MESHING: bool = true;

#[derive(Debug, Clone)]
pub struct Buffer {
    pub buffer: Vec<BlockVertex>,
    pub index_buffer: Vec<u32>,
}

impl Buffer {
    #[inline]
    pub fn new() -> Self { Self { buffer: vec![], index_buffer: vec![] }}

    #[inline]
    fn push_line(&mut self, vertices: &[BlockVertex; 4], indices: &[usize; 6]) {
        let mut index_vertex: [Option<usize>; 4] = [None, None, None, None];
        indices.iter().for_each(|i| {
            let current_index = self.buffer.len();
            index_vertex[*i] = Some(current_index);
            self.buffer.push(vertices[*i]);
            self.index_buffer.push(current_index as u32);
            if *i != 0 && *i < indices.len() - 1 {
                self.buffer.push(vertices[*i]);
                self.index_buffer.push(current_index as u32);
            }
        });
    }

    pub fn manage_vertices(&mut self, vertices: &[BlockVertex; 4], indices: &[usize; 6]) {
        if !IS_LINE_TOPOLOGY {
            self.push_triangles(vertices, indices);
        } else {
            self.push_line(vertices, indices);
        }
    }

    #[inline]
    fn push_triangles(&mut self, vertices: &[BlockVertex], indices: &[usize; 6]) {
        let mut index_vertex: [Option<usize>; 4] = [None, None, None, None];
        indices.iter().for_each(|i| {
            if let Some(index_vertex) = index_vertex[*i] {
                self.index_buffer.push(index_vertex as u32);
                return;
            }
            let current_index = self.buffer.len();
            index_vertex[*i] = Some(current_index);
            self.buffer.push(vertices[*i]);
            self.index_buffer.push(current_index as u32);
        });
    }
}

pub struct RenderResult {
    pub coord: ChunkCoord,
    pub mesh: MeshInput,
}

pub fn render(cc: ChunkCoord, chunks: &Chunks, content: &Content) -> Option<RenderResult> {
    println!("txasxax1");
    let Some(chunk) = unsafe {&*chunks.chunks.get()}.get(&cc).cloned() else {return None};
    println!("txasxax2");
    let mut models = Models::new();
    let mut animated_models = AnimatedModels::new();
    
    let mut block_manager = BlockManagers::new(!IS_GREEDY_MESHING);
    let mut glass_manager = BlockManagers::new(!IS_GREEDY_MESHING);
    
    let mut buffer = Buffer::new();
    let mut glass_buffer = Buffer::new();
    let mut belt_buffer = Buffer::new();

    for (ly, lz, lx) in iproduct!(0..CHUNK_SIZE, 0..WORLD_BLOCK_HEIGHT, 0..CHUNK_SIZE) {
        let id = unsafe {chunk.voxels().get_unchecked((lx, ly, lz).into()).id()};
        if id == 0 { continue };
        let block = &content.blocks[id as usize];
        match block.block_type() {
            BlockType::Block {faces} => {
                if block.is_glass() {
                    render_block(content, &mut glass_manager, chunks, &chunk, &block.base, faces, (lx, ly, lz));
                } else {
                    render_block(content, &mut block_manager, chunks, &chunk, &block.base, faces, (lx, ly, lz));
                }
            },
            BlockType::None => {},
            BlockType::Model {id} => {
                render_model(&mut models, chunk.as_ref(), *id, lx, ly, lz);
            },
            BlockType::AnimatedModel {id} => {
                render_animated_model(&mut animated_models, chunk.as_ref(), *id, lx, ly, lz);
            },
            BlockType::ComplexObject {id} => {
                let complex_object = &chunks.content.complex_objects[*id as usize];
                render_complex_object(complex_object, &mut models, &mut animated_models, &mut buffer, &mut belt_buffer, chunk.as_ref(), lx, ly, lz);
            },
        };
    }
    let global = chunk.coord.to_global(LocalCoord::new(0, 0, 0)).into();
    block_manager.manage_vertices(&mut buffer, global);
    glass_manager.manage_vertices(&mut glass_buffer, global);
    Some(RenderResult {
        coord: chunk.coord,
        mesh: MeshInput {
            block_vertices: buffer.buffer,
            block_indices: buffer.index_buffer,
            glass_vertices: glass_buffer.buffer,
            glass_indices: glass_buffer.index_buffer,
            models,
            animated_models,
            belt_vertices: belt_buffer.buffer,
            belt_indices: belt_buffer.index_buffer,
        }
    })
}