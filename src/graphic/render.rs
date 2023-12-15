use std::collections::HashMap;

use crate::{voxels::{chunk::CHUNK_SIZE, chunks::Chunks, block::{blocks::BLOCKS, block_type::BlockType, light_permeability::LightPermeability}}, engine::vertices::block_vertex::BlockVertex, world::{World, chunk_coords::ChunkCoords}, engine::pipeline::IS_LINE};

use super::complex_object::{ComplexObjectPart, ComplexObjectSide};

#[derive(Clone, Copy)]
enum Axis {X = 0, Y = 1, Z = 2}
impl Axis {
    pub fn to_usize(self) -> usize {self as usize}
}

enum Direction{Top, Left}


// Fix pixel gaps 
//     More number => less pixel gaps
//     Smaller number => more artifacts associated with block enlargement
// https://stackoverflow.com/questions/39958039/where-do-pixel-gaps-come-from-in-opengl
const STITCHING_COEFFICIENT: f32 = 0.0014; //0.0025

type GreedyVertices = [[Vec<[BlockVertex; 4]>; CHUNK_SIZE]; CHUNK_SIZE];
struct GreedMesh {
    greed_vertices: GreedyVertices,
}
impl GreedMesh {
    pub fn new() -> Self { Self {greed_vertices: Default::default()} }

    pub fn push(&mut self, i0: usize, i1: usize, points: [BlockVertex; 4]) {
        self.greed_vertices[i0][i1].push(points);
    }

    pub fn greed(&mut self, buffer: &mut Buffer, axis: &Axis, triangle_indices: &[usize]) {
        self.raw_greed(axis, Direction::Top);
        self.raw_greed(axis, Direction::Left);

        match axis {
            Axis::X => {
                self.greed_vertices.iter_mut().for_each(|gv| {
                    gv.iter_mut().for_each(|v| {
                        v.iter_mut().for_each(|vertices| {
                            vertices[0].position[1] -= STITCHING_COEFFICIENT;
                            vertices[0].position[2] -= STITCHING_COEFFICIENT;

                            vertices[1].position[1] += STITCHING_COEFFICIENT;
                            vertices[1].position[2] -= STITCHING_COEFFICIENT;

                            vertices[2].position[1] += STITCHING_COEFFICIENT;
                            vertices[2].position[2] += STITCHING_COEFFICIENT;

                            vertices[3].position[1] -= STITCHING_COEFFICIENT;
                            vertices[3].position[2] += STITCHING_COEFFICIENT;

                        });
                    });
                });
            },
            Axis::Y => {
                self.greed_vertices.iter_mut().for_each(|gv| {
                    gv.iter_mut().for_each(|v| {
                        v.iter_mut().for_each(|vertices| {
                            vertices[0].position[0] -= STITCHING_COEFFICIENT;
                            vertices[0].position[2] -= STITCHING_COEFFICIENT;

                            vertices[1].position[0] -= STITCHING_COEFFICIENT;
                            vertices[1].position[2] += STITCHING_COEFFICIENT;

                            vertices[2].position[0] += STITCHING_COEFFICIENT;
                            vertices[2].position[2] += STITCHING_COEFFICIENT;

                            vertices[3].position[0] += STITCHING_COEFFICIENT;
                            vertices[3].position[2] -= STITCHING_COEFFICIENT;
                        });
                    });
                });
            },
            Axis::Z => {
                self.greed_vertices.iter_mut().for_each(|gv| {
                    gv.iter_mut().for_each(|v| {
                        v.iter_mut().for_each(|vertices| {
                            vertices[0].position[0] -= STITCHING_COEFFICIENT;
                            vertices[0].position[1] -= STITCHING_COEFFICIENT;

                            vertices[1].position[0] += STITCHING_COEFFICIENT;
                            vertices[1].position[1] -= STITCHING_COEFFICIENT;

                            vertices[2].position[0] += STITCHING_COEFFICIENT;
                            vertices[2].position[1] += STITCHING_COEFFICIENT;
                            
                            vertices[3].position[0] -= STITCHING_COEFFICIENT;
                            vertices[3].position[1] += STITCHING_COEFFICIENT;

                        });
                    });
                });
            },
        }
        buffer.manage_greedy_vertices(&self.greed_vertices, triangle_indices);
    }


    fn raw_greed(&mut self, axis: &Axis, direction: Direction) {
        let uv_index: usize = match direction {
            Direction::Top => 0,
            Direction::Left => 1,
        };
        let p_indices: [usize; 2] = match axis {
            Axis::X => [1, 2],
            Axis::Y => [0, 2],
            Axis::Z => [0, 1],
        };
        let indices0: [usize; 2] = match direction {
            Direction::Top => [0, 1],
            Direction::Left => [0, 3],
        };
        let indices1: [usize; 2] = match direction {
            Direction::Top => [3, 2],
            Direction::Left => [1, 2],
        };
        let prev_ind = match direction {
            Direction::Top => [0, 1],
            Direction::Left => [1, 0],
        };
        for i in prev_ind[0]..CHUNK_SIZE {
            for j in prev_ind[1]..CHUNK_SIZE {
                for k in 0..self.greed_vertices[i][j].len() {
                    let mut point0 = self.greed_vertices[i][j][k];
                    for w in 0..self.greed_vertices[i-prev_ind[0]][j-prev_ind[1]].len() {
                        let point1 = self.greed_vertices[i-prev_ind[0]][j-prev_ind[1]][w];
                        let side = point0[0].position[axis.to_usize()] == point1[0].position[axis.to_usize()];
                        let layer = point1[0].layer == point0[0].layer;
    
                        let x_bottom = point1[indices1[0]].position[p_indices[0]] == point0[indices0[0]].position[p_indices[0]];
                        let z_bottom = point1[indices1[0]].position[p_indices[1]] == point0[indices0[0]].position[p_indices[1]];
                        let x_top = point1[indices1[1]].position[p_indices[0]] == point0[indices0[1]].position[p_indices[0]];
                        let z_top = point1[indices1[1]].position[p_indices[1]] == point0[indices0[1]].position[p_indices[1]];

                        let light = point0.iter().zip(point1.iter()).all(|points| {
                            points.0.v_light == points.1.v_light
                        });
    
                        if side && x_bottom && z_bottom && x_top && z_top && layer && light {
                            point0[indices0[0]].position = point1[indices0[0]].position;
                            point0[indices0[1]].position = point1[indices0[1]].position;
                            
                            point0[1].uv[uv_index] += point1[1].uv[uv_index];
                            point0[2].uv[uv_index] += point1[2].uv[uv_index];
                            point0[3].uv[uv_index] += point1[3].uv[uv_index];
    
                            self.greed_vertices[i-prev_ind[0]][j-prev_ind[1]].remove(w);
                            self.greed_vertices[i][j][k] = point0;
                        }
                    }
                }
            }
        }
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

    pub fn push_vertex(&mut self, x: f32, y: f32, z: f32, u: f32, v: f32, layer: f32, light: &[f32]) -> u16 {
        let current_index = self.buffer.len() as u16;
        self.buffer.push(
            BlockVertex {
                position: [x, y, z],
                uv: [u, v],
                layer: layer as u32,
                v_light: [light[0], light[1], light[2], light[3]]});
        self.index_buffer.push(current_index);
        current_index
    }

    pub fn manage_greedy_vertices(&mut self, greedy_vertices: &GreedyVertices, indices: &[usize]) {
        greedy_vertices.iter().for_each(|column| {
            column.iter().for_each(|item| {
                item.iter().for_each(|vertices| {
                    if !IS_LINE {
                        self.push_triangles(vertices, indices);
                    } else {
                        self.push_line(vertices, indices);
                    }
                    
                });
            });
        });
    }

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
    // TODO
    // Fix small pixel gaps
    // https://stackoverflow.com/questions/39958039/where-do-pixel-gaps-come-from-in-opengl
    let chunks = &world.chunks;
    let mut buffer = Buffer::new();

    let mut transport_belt_buffer = Buffer::new();
    let light_handler = LightHandler::new(chunks);
    let Some(Some(chunk)) = chunks.chunks.get(chunk_index).map(|c| c.as_ref()) else {return None};
    let mut models = HashMap::<String, Vec<ModelRenderResult>>::new();
    let mut animated_models_data = HashMap::<String, Vec<AnimatedModelRenderResult>>::new();
    for ly in 0..CHUNK_SIZE {
        let mut greedy_vertices_top = GreedMesh::new();
        let mut greedy_vertices_bottom = GreedMesh::new();
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let id = chunk.voxel((lx, ly, lz).into()).id;
                if id == 0 { continue };
                let (x, y, z) = chunk.xyz.to_global((lx, ly, lz).into()).into();
                
                let block = &BLOCKS()[id as usize];
                
                let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
                let (gmx, gpx) = (x as f32, x as f32+1.0);
                let (gmy, gpy) = (y as f32, y as f32+1.0);
                let (gmz, gpz) = (z as f32, z as f32+1.0);
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

                if !is_blocked(x, y+1, z, chunks, LightPermeability::UP, block.light_permeability()) {
                    let layer = faces[3] as f32;

                    let ld = light_handler.light_default((x, py, z));
                    let l0 = light_handler.light_numbered(&ld, (nx,py,z), (nx,py,nz), (x,py,nz));
                    let l1 = light_handler.light_numbered(&ld, (nx,py,z), (nx,py,pz), (x,py,pz));
                    let l2 = light_handler.light_numbered(&ld, (px,py,z), (px,py,pz), (x,py,pz));
                    let l3 = light_handler.light_numbered(&ld, (px,py,z), (px,py,nz), (x,py,nz));

                    buffer.push_triangles(&[
                        get_block_vertex(gmx, gpy, gmz, 0., 0., layer, &l0),
                        get_block_vertex(gmx, gpy, gpz, 0., 1., layer, &l1),
                        get_block_vertex(gpx, gpy, gpz, 1., 1., layer, &l2),
                        get_block_vertex(gpx, gpy, gmz, 1., 0., layer, &l3)
                    ], &[3,2,0,2,1,0]);
                }
                if !is_blocked(x, y-1, z, chunks, LightPermeability::DOWN, block.light_permeability()) {
                    let layer = faces[2] as f32;
            
                    let ld = light_handler.light_default((x, y-1, z));
                    let l0 = light_handler.light_numbered(&ld, (x-1, y-1, z-1), (x-1, y-1, z), (x, y-1, z-1));
                    let l1 = light_handler.light_numbered(&ld, (x+1, y-1, z+1), (x+1, y-1, z), (x, y-1, z+1));
                    let l2 = light_handler.light_numbered(&ld, (x-1, y-1, z+1), (x-1, y-1, z), (x, y-1, z+1));
                    let l3 = light_handler.light_numbered(&ld, (x+1, y-1, z-1), (x+1, y-1, z), (x, y-1, z-1));
            
                    greedy_vertices_bottom.push(lz, lx, [
                        get_block_vertex(gmx, gmy, gmz, 0., 0., layer, &l0),
                        get_block_vertex(gmx, gmy, gpz, 0., 1., layer, &l2),
                        get_block_vertex(gpx, gmy, gpz, 1., 1., layer, &l1),
                        get_block_vertex(gpx, gmy, gmz, 1., 0., layer, &l3),
                    ]);
                }
            }
            
        }
        greedy_vertices_top.greed(&mut buffer, &Axis::Y, &[3,2,0,2,1,0]);
        greedy_vertices_bottom.greed(&mut buffer, &Axis::Y, &[0,1,2,0,2,3]);
    }


    for lx in 0..CHUNK_SIZE {
        let mut greedy_vertices_right = GreedMesh::new();
        let mut greedy_vertices_left = GreedMesh::new();
        for ly in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let id = chunk.voxel((lx, ly, lz).into()).id;
                if id == 0 { continue };
                let (x, y, z) = chunk.xyz.to_global((lx, ly, lz).into()).into();

                let block = &BLOCKS()[id as usize];
                
                let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
                let (gmx, gpx) = (x as f32, x as f32+1.0);
                let (gmy, gpy) = (y as f32, y as f32+1.0);
                let (gmz, gpz) = (z as f32, z as f32+1.0);
                let faces = match block.block_type() {
                    BlockType::Block {faces} => faces,
                    _ => continue
                };
                if !is_blocked(x+1, y, z, chunks, LightPermeability::RIGHT, block.light_permeability()) {
                    let layer = faces[1] as f32;

                    let ld = light_handler.light_default((px, y, z));
                    let l0 = light_handler.light_numbered(&ld, (px,ny,nz), (px,y,nz), (px,ny,z));
                    let l1 = light_handler.light_numbered(&ld, (px,py,nz), (px,y,nz), (px,py,z));
                    let l2 = light_handler.light_numbered(&ld, (px,py,pz), (px,y,pz), (px,py,z));
                    let l3 = light_handler.light_numbered(&ld, (px,ny,pz), (px,y,pz), (px,ny,z));

                    greedy_vertices_right.push(ly, lz, [
                        get_block_vertex(gpx, gmy, gmz, 0., 0., layer, &l0),
                        get_block_vertex(gpx, gpy, gmz, 0., 1., layer, &l1),
                        get_block_vertex(gpx, gpy, gpz, 1., 1., layer, &l2),
                        get_block_vertex(gpx, gmy, gpz, 1., 0., layer, &l3)
                    ]);
                }
                if !is_blocked(x-1, y, z, chunks, LightPermeability::LEFT, block.light_permeability()) {
                    let layer = faces[0] as f32;
            
                    let ld = light_handler.light_default((nx, y, z));
                    let l0 = light_handler.light_numbered(&ld, (nx,ny,nz), (nx,y,nz), (nx,ny,z));
                    let l1 = light_handler.light_numbered(&ld, (nx,py,pz), (nx,y,pz), (nx,py,z));
                    let l2 = light_handler.light_numbered(&ld, (nx,py,nz), (nx,y,nz), (nx,py,z));
                    let l3 = light_handler.light_numbered(&ld, (nx,ny,pz), (nx,y,pz), (nx,ny,z));
            
                    //change light l2 -> l1
                    greedy_vertices_left.push(ly, lz, [
                        get_block_vertex(gmx, gmy, gmz, 0., 0., layer, &l0),
                        get_block_vertex(gmx, gpy, gmz, 0., 1., layer, &l2),
                        get_block_vertex(gmx, gpy, gpz, 1., 1., layer, &l1),
                        get_block_vertex(gmx, gmy, gpz, 1., 0., layer, &l3)
                    ]);
                }
            }
            
        }

        greedy_vertices_right.greed(&mut buffer, &Axis::X, &[3,2,0,2,1,0]);
        greedy_vertices_left.greed(&mut buffer, &Axis::X, &[0,1,2,0,2,3]);
    }


    for lz in 0..CHUNK_SIZE {
        let mut greedy_vertices_front = GreedMesh::new();
        let mut greedy_vertices_back = GreedMesh::new();
        for lx in 0..CHUNK_SIZE {
            for ly in 0..CHUNK_SIZE {
                let id = chunk.voxel((lx, ly, lz).into()).id;
                if id == 0 { continue };
                let (x, y, z) = chunk.xyz.to_global((lx, ly, lz).into()).into();

                let block = &BLOCKS()[id as usize];
                
                let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
                let (gmx, gpx) = (x as f32, x as f32+1.0);
                let (gmy, gpy) = (y as f32, y as f32+1.0);
                let (gmz, gpz) = (z as f32, z as f32+1.0);
                let faces = match block.block_type() {
                    BlockType::Block {faces} => faces,
                    _ => continue
                };
                if !is_blocked(x, y, z+1, chunks, LightPermeability::FRONT, block.light_permeability()) {
                    let layer = faces[5] as f32;

                    let ld = light_handler.light_default((x, y, pz));
                    let l0 = light_handler.light_numbered(&ld, (nx,ny,pz), (x,ny,pz), (nx,y,pz));
                    let l1 = light_handler.light_numbered(&ld, (px,py,pz), (x,py,pz), (px,y,pz));
                    let l2 = light_handler.light_numbered(&ld, (nx,py,pz), (x,py,pz), (nx,y,pz));
                    let l3 = light_handler.light_numbered(&ld, (px,ny,pz), (x,ny,pz), (px,y,pz));

                    greedy_vertices_front.push(lx, ly, [
                        get_block_vertex(gmx, gmy, gpz, 0., 0., layer, &l0),
                        get_block_vertex(gpx, gmy, gpz, 0., 1., layer, &l3),
                        get_block_vertex(gpx, gpy, gpz, 1., 1., layer, &l1),
                        get_block_vertex(gmx, gpy, gpz, 1., 0., layer, &l2),
                    ]);
                }
                if !is_blocked(x, y, z-1, chunks, LightPermeability::BACK, block.light_permeability()) {
                    let layer = faces[4] as f32;
            
                    let ld = light_handler.light_default((x, y, nz));
                    let l0 = light_handler.light_numbered(&ld, (nx,ny,nz), (x,ny,nz), (nx,y,nz));
                    let l1 = light_handler.light_numbered(&ld, (nx,py,nz), (x,py,nz), (nx,y,nz));
                    let l2 = light_handler.light_numbered(&ld, (px,py,nz), (x,py,nz), (px,y,nz));
                    let l3 = light_handler.light_numbered(&ld, (px,ny,nz), (x,ny,nz), (px,y,nz));
            
                    greedy_vertices_back.push(lx, ly, [
                        get_block_vertex(gmx, gmy, gmz, 0., 0., layer, &l0),
                        get_block_vertex(gpx, gmy, gmz, 0., 1., layer, &l3),
                        get_block_vertex(gpx, gpy, gmz, 1., 1., layer, &l2),
                        get_block_vertex(gmx, gpy, gmz, 1., 0., layer, &l1),
                    ]);
                }
            }
        }

        greedy_vertices_front.greed(&mut buffer, &Axis::Z, &[3,2,0,2,1,0]);
        greedy_vertices_back.greed(&mut buffer, &Axis::Z, &[0,1,2,0,2,3]);
    }

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
    const COEFFICIENT: f32 = 30.0;
    pub fn new(chunks: &'a Chunks) -> Self { Self { chunks }}

    pub fn light_default(&self, face: (i32, i32, i32)) -> Vec<f32> {
        (0..4).map(|item| {
            self.chunks.light((face.0,face.1,face.2).into(),item) as f32/15.0
        }).collect::<Vec<f32>>()
    }

    pub fn light_numbered(&self, light_default: &[f32], c1: (i32, i32, i32), c2: (i32, i32, i32), c3: (i32, i32, i32)) -> Vec<f32> {
        light_default.iter().enumerate().map(|(i, light)| {
            (self.chunks.light((c1.0,c1.1,c1.2).into(),i as u8) as f32 +
             self.chunks.light((c2.0,c2.1,c2.2).into(),i as u8) as f32 +
             self.chunks.light((c3.0,c3.1,c3.2).into(),i as u8) as f32 +
             light*Self::COEFFICIENT) / 75.0
        }).collect::<Vec<f32>>()
    }
}

fn is_blocked(x: i32, y: i32, z: i32, chunks: &Chunks, side: LightPermeability, current: LightPermeability) -> bool {
    if let Some(voxel) = chunks.voxel_global((x, y, z).into()) {
        let block = &BLOCKS()[voxel.id as usize];
        return((block.light_permeability() & side.get_opposite_side()).bits() == 0) && ((current & side).bits() == 0)
    }
    false
}
