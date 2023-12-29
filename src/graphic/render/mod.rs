use std::collections::HashMap;

use itertools::iproduct;

use crate::{voxels::{chunk::CHUNK_SIZE, chunks::Chunks, block::{blocks::BLOCKS, block_type::BlockType, light_permeability::LightPermeability}}, engine::vertices::block_vertex::BlockVertex, world::{World, chunk_coords::ChunkCoords}, engine::pipeline::IS_LINE, graphic::render::block_managers::BlockManagers};
use crate::light::light_map::Light;
use super::complex_object::{ComplexObjectPart, ComplexObjectSide};

pub mod block_managers;

const IS_GREEDY_MESHING: bool = true;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockFaceLight([Light; 9]);

impl BlockFaceLight {
    pub fn new(chunks: &Chunks, coords: [(i32, i32, i32); 9]) -> Self {
        Self(coords.map(|coord| chunks.get_light(coord.into())))
    }

    const ANGLE_INDICES: [[usize; 3]; 4] = [
        [1,   0,   3],
        [1,   2,   5],
        [7,   8,   5],
        [7,   6,   3]
    ];
    const CHANNELS: [u8; 4] = [0, 1, 2, 3];
    const CENTER_INFLUENCE: f32 = 1.5;
    const MAX_LIGHT_COUNT: f32 = 15.0;
    const COEFFICIENT: f32 = Self::MAX_LIGHT_COUNT * (Self::CENTER_INFLUENCE + 3.0);
    pub fn get(&self) -> [[f32; 4]; 4] {
        let center = self.0[4];
        Self::ANGLE_INDICES.map(|inds| {
            Self::CHANNELS.map(|i| {unsafe {
                (self.0.get_unchecked(inds[0]).get(i) as f32 +
                 self.0.get_unchecked(inds[1]).get(i) as f32 +
                 self.0.get_unchecked(inds[2]).get(i) as f32 +
                 (center.get(i) as f32 * Self::CENTER_INFLUENCE)) / Self::COEFFICIENT
            }})
        })
    }
}

#[derive(Debug)]
pub struct BlockFace {
    layer: u32,
    light: BlockFaceLight,
    size: [u8; 2],
}

impl BlockFace {
    pub fn new(layer: u32, light: BlockFaceLight) -> Self {
        Self { layer, light, size: [1, 1] }
    }
}


#[derive(Debug)]
pub struct Buffer {
    pub buffer: Vec<BlockVertex>,
    pub index_buffer: Vec<u16>,
}


fn get_block_vertex(x: f32, y: f32, z: f32, u: f32, v: f32, layer: f32, light: &[f32]) -> BlockVertex {
    BlockVertex {
        position: [x, y, z],
        uv: [u, v],
        layer: layer as u32,
        v_light: [light[0], light[1], light[2], light[3]]}
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

fn push_complex_vertices(
  buffer: &mut Buffer,
  light_handler: &LightHandler<'_>,
  side: &ComplexObjectSide,
  indices: &[usize],
  xyz: (i32, i32, i32),
  rotation_index: usize,
) {
    let layer = side.texture_layer as f32;
    let ld = light_handler.light_default((xyz.0, xyz.1, xyz.2));
    side.vertex_groups.iter().for_each(|group| {
        let vertices: Vec<BlockVertex> = (0..4).map(|i| {
            get_block_vertex(
                xyz.0 as f32 + group.x(rotation_index, i),
                xyz.1 as f32 + group.y(rotation_index, i),
                xyz.2 as f32 + group.z(rotation_index, i),
                group.u(i),
                group.v(i),
                layer,
                &ld
            )
        }).collect();
        buffer.push_triangles(&vertices, indices);
    }); 
}

fn render_complex_side(
    part: &ComplexObjectPart,
    buffer: &mut Buffer,
    light_handler: &LightHandler<'_>,
    xyz: (i32, i32, i32),
    rotation_index: usize
) {
    [(&part.positive_y, [3,2,0,2,1,0]), (&part.negative_y, [0,1,2,0,2,3]),
     (&part.positive_x, [3,2,0,2,1,0]), (&part.negative_x, [0,1,2,0,2,3]),
     (&part.positive_z, [3,2,0,2,1,0]), (&part.negative_z, [0,1,2,0,2,3])]
        .iter().for_each(|(side, indices)| {
            push_complex_vertices(buffer, light_handler, side, indices, xyz, rotation_index);
        });
}

#[derive(Debug)]
pub struct ModelRenderResult {
    pub position: [f32; 3],
    pub light: [f32; 4],
    pub rotation_index: u32,
}

#[derive(Debug)]
pub struct AnimatedModelRenderResult {
    pub position: [f32; 3],
    pub light: [f32; 4],
    pub progress: f32,
    pub rotation_index: u32,
}

#[derive(Debug)]
pub struct RenderResult {
    pub chunk_index: usize,
    pub xyz: ChunkCoords,
    pub block_vertices: Vec<BlockVertex>,
    pub block_indices: Vec<u16>,
    pub belt_vertices: Vec<BlockVertex>,
    pub belt_indices: Vec<u16>,

    pub models: HashMap<String, Vec<ModelRenderResult>>,
    pub animated_models: HashMap<String, Vec<AnimatedModelRenderResult>>,
}

pub fn render(chunk_index: usize, world: &World) -> Option<RenderResult> {
    let chunks = &world.chunks;
    let mut transport_belt_buffer = Buffer::new();
    let light_handler = LightHandler::new(chunks);
    let Some(Some(chunk)) = chunks.chunks.get(chunk_index).map(|c| c.as_ref()) else {return None};
    let mut models = HashMap::<String, Vec<ModelRenderResult>>::new();
    let mut animated_models_data = HashMap::<String, Vec<AnimatedModelRenderResult>>::new();

    let mut block_manager = BlockManagers::new(IS_GREEDY_MESHING);
    let mut buffer = Buffer::new();

    for (ly, lz, lx) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
        let id = unsafe {chunk.get_unchecked_voxel((lx, ly, lz).into()).id};
        if id == 0 { continue };
        let (x, y, z) = chunk.xyz.to_global((lx, ly, lz).into()).into();
        
        let block = &BLOCKS()[id as usize];
        
        let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
        let faces = match block.block_type() {
            BlockType::Block {faces} => faces,
            BlockType::None => {continue},
            BlockType::Model {name} => {
                let rotation_index = chunk.voxels_data.get(&((ly*CHUNK_SIZE+lz)*CHUNK_SIZE+lx))
                    .and_then(|vd| vd.rotation_index()).unwrap_or(0);
                let ld = light_handler.light_default((x, y, z));
                let data = ModelRenderResult {
                    position: [x as f32, y as f32, z as f32],
                    light: [ld[0], ld[1], ld[2], ld[3]],
                    rotation_index
                };
                if models.contains_key(name) {
                    models.get_mut(name).unwrap().push(data);
                } else {
                    models.insert(name.to_string(), vec![data]);
                }
                // Need to copy data
                // models.entry(name.to_owned())
                //     .and_modify(|v| v.push(data))
                //     .or_insert(vec![data]);
                continue;
            },
            BlockType::AnimatedModel {name} => {
                let e = chunk.voxels_data.get(&((ly*CHUNK_SIZE+lz)*CHUNK_SIZE+lx)).unwrap();
                let progress = e.additionally.animation_progress().expect("No animation progress");
                let rotation_index = e.rotation_index().unwrap_or(0);
                let ld = light_handler.light_default((x, y, z));
                let data = AnimatedModelRenderResult {
                    position: [x as f32, y as f32, z as f32],
                    light: [ld[0], ld[1], ld[2], ld[3]],
                    progress,
                    rotation_index
                };
                if animated_models_data.contains_key(name) {
                    animated_models_data.get_mut(name).unwrap().push(data);
                } else {
                    animated_models_data.insert(name.to_string(), vec![data]);
                }
                // Need to copy data
                // animated_models_data.entry(name.to_owned())
                //     .and_modify(|v| v.push(data))
                //     .or_insert(vec![data]);
                continue;
            },
            BlockType::ComplexObject {cp} => {
                let vd = chunk.voxel_data((lx, ly, lz).into()).unwrap();
                let ri = vd.rotation_index().unwrap_or(0) as usize;
                cp.parts.iter().for_each(|parts| {
                    match parts {
                        super::complex_object::ComplexObjectParts::Block(part) => {
                            render_complex_side(part, &mut buffer, &light_handler, (x, y, z), ri);
                        },
                        super::complex_object::ComplexObjectParts::TransportBelt(part) => {
                            render_complex_side(part, &mut transport_belt_buffer, &light_handler, (x, y, z), ri);
                        },
                    }
                });
                
                continue;
            },
        };

        if !is_blocked(x-1, y, z, chunks, LightPermeability::LEFT, block.light_permeability()) {
            let light = BlockFaceLight::new(chunks, [
                (nx, ny, nz), (nx, y, nz), (nx, py, nz),
                (nx, ny,  z), (nx, y, z),  (nx, py, z),
                (nx, ny, pz), (nx, y, pz), (nx, py, pz)
            ]);
            block_manager.set(0, lx, ly, lz, BlockFace::new(faces[0], light));
        }

        if !is_blocked(x+1, y, z, chunks, LightPermeability::RIGHT, block.light_permeability()) {
            let light = BlockFaceLight::new(chunks, [
                (px, ny, nz), (px, y, nz), (px, py, nz),
                (px, ny,  z), (px, y, z),  (px, py, z),
                (px, ny, pz), (px, y, pz), (px, py, pz)
            ]);
            block_manager.set(1, lx, ly, lz, BlockFace::new(faces[1], light));
        }

        if !is_blocked(x, y-1, z, chunks, LightPermeability::DOWN, block.light_permeability()) {
            let light = BlockFaceLight::new(chunks, [
                (nx, ny, nz), (nx, ny, z), (nx, ny, pz),
                (x,  ny, nz), (x,  ny, z), (x,  ny, pz),
                (px, ny, nz), (px, ny, z), (px, ny, pz)
            ]);
            block_manager.set(2, ly, lx, lz, BlockFace::new(faces[2], light));
        }


        if !is_blocked(x, y+1, z, chunks, LightPermeability::UP, block.light_permeability()) {
            let light = BlockFaceLight::new(chunks, [
                (nx, py, nz), (nx, py, z), (nx, py, pz),
                (x,  py, nz), (x,  py, z), (x,  py, pz),
                (px, py, nz), (px, py, z), (px, py, pz)
            ]);
            block_manager.set(3, ly, lx, lz, BlockFace::new(faces[3], light));
        }

        if !is_blocked(x, y, z-1, chunks, LightPermeability::DOWN, block.light_permeability()) {
            let light = BlockFaceLight::new(chunks, [
                (nx, ny, nz), (x, ny, nz), (px, ny, nz),
                (nx,  y, nz), (x,  y, nz), (px,  y, nz),
                (nx, py, nz), (x, py, nz), (px, py, nz)
            ]);
            block_manager.set(4, lz, lx, ly, BlockFace::new(faces[4], light));
        }

        if !is_blocked(x, y, z+1, chunks, LightPermeability::UP, block.light_permeability()) {
            let light = BlockFaceLight::new(chunks, [
                (nx, ny, pz), (x, ny, pz), (px, ny, pz),
                (nx,  y, pz), (x,  y, pz), (px,  y, pz),
                (nx, py, pz), (x, py, pz), (px, py, pz)
            ]);
            block_manager.set(5, lz, lx, ly, BlockFace::new(faces[5], light));
        }
    }
    let global = chunk.xyz.to_global((0u8, 0, 0).into()).into();
    block_manager.manage_vertices(&mut buffer, global);
    Some(RenderResult {
        chunk_index,
        xyz: chunk.xyz,
        block_vertices: buffer.buffer,
        block_indices: buffer.index_buffer,
        models,
        animated_models: animated_models_data,
        belt_vertices: transport_belt_buffer.buffer,
        belt_indices: transport_belt_buffer.index_buffer,
    })
}


struct LightHandler<'a> {
    chunks: &'a Chunks,
}

impl<'a> LightHandler<'a> {
    pub fn new(chunks: &'a Chunks) -> Self { Self { chunks }}
    pub fn light_default(&self, face: (i32, i32, i32)) -> Vec<f32> {
        (0..4).map(|item| {
            self.chunks.light((face.0,face.1,face.2).into(),item) as f32/15.0
        }).collect::<Vec<f32>>()
    }
}

#[inline]
fn is_blocked(x: i32, y: i32, z: i32, chunks: &Chunks, side: LightPermeability, current: LightPermeability) -> bool {
    let Some(voxel) = chunks.voxel_global((x, y, z).into()) else {return false};
    let block = &BLOCKS()[voxel.id as usize];
    ((block.light_permeability() & side.get_opposite_side()).bits() == 0) && ((current & side).bits() == 0)
}