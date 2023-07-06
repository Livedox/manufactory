#[derive(Clone, Copy)]
enum Axis {X = 0, Y = 1, Z = 2}
impl Axis {
    pub fn to_usize(&self) -> usize {*self as usize}
}

enum Direction{Top, Left}

use crate::{voxels::{chunk::{CHUNK_SIZE, CHUNK_SQUARE, CHUNK_VOLUME}, chunks::Chunks, block::{BLOCKS, Block, LightPermeability}}, vertices::block_vertex::BlockVertex, state::IS_LINE};

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
                        // if w >= self.greed_vertices[i-prev_ind[0]][j-prev_ind[1]].len() { continue; }
                        let point1 = self.greed_vertices[i-prev_ind[0]][j-prev_ind[1]][w];
                        let side = point0[0].position[axis.to_usize()] == point1[0].position[axis.to_usize()];
                        let layer = point1[0].layer == point0[0].layer;
    
                        let x_bottom = point1[indices1[0]].position[p_indices[0]] == point0[indices0[0]].position[p_indices[0]];
                        let z_bottom = point1[indices1[0]].position[p_indices[1]] == point0[indices0[0]].position[p_indices[1]];
                        let x_top = point1[indices1[1]].position[p_indices[0]] == point0[indices0[1]].position[p_indices[0]];
                        let z_top = point1[indices1[1]].position[p_indices[1]] == point0[indices0[1]].position[p_indices[1]];
                    
                        let light_bottom = point1[indices1[0]].v_light == point0[indices0[0]].v_light;
                        let light_top = point1[indices1[1]].v_light == point0[indices0[1]].v_light;
    
                        if side && x_bottom && z_bottom && x_top && z_top && layer && light_bottom && light_top {
                            point0[indices0[0]].position = point1[indices0[0]].position;
                            point0[indices0[1]].position = point1[indices0[1]].position;
                            
                            
                            // point0[indices1[0]].uv[uv_index] += point1[indices1[0]].uv[uv_index];
                            // point0[indices1[1]].uv[uv_index] += point1[indices1[1]].uv[uv_index];
                            
                            // point0[0].uv[uv_index] += point1[0].uv[uv_index];
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
        layer: layer,
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
                layer: layer,
                v_light: [light[0], light[1], light[2], light[3]]});
        self.index_buffer.push(current_index);
        current_index
    }

    pub fn manage_greedy_vertices(&mut self, greedy_vertices: &GreedyVertices, indices: &[usize]) {
        greedy_vertices.iter().for_each(|column| {
            column.iter().for_each(|item| {
                item.iter().for_each(|vertices| {
                    if !IS_LINE {
                        self.push_triangle(vertices, indices);
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

    pub fn push_triangle(&mut self, vertices: &[BlockVertex; 4], indices: &[usize]) {
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

pub struct VoxelRenderer {}
impl VoxelRenderer {
    // pub fn render(&mut self, chunk_index: usize, chunks: &mut Chunks) -> (Vec<BlockVertex>, Vec<u16>) {
    //     let chunks = Arc::new(chunks);
    //     let mut buffer = Buffer::new();
    //     for local_y in 0..CHUNK_SIZE {
    //         for local_z in 0..CHUNK_SIZE {
    //             for local_x in 0..CHUNK_SIZE {
    //                 let chunk = chunks.chunks[chunk_index].as_ref().unwrap();
    //                 let id = chunk.voxel((local_x, local_y, local_z)).id;
    //                 if id == 0 { continue };
    //                 let (x, y, z) = (
    //                     (chunk.chunk_coords.0*CHUNK_SIZE as i32 + local_x as i32),
    //                     (chunk.chunk_coords.1*CHUNK_SIZE as i32 + local_y as i32),
    //                     (chunk.chunk_coords.2*CHUNK_SIZE as i32 + local_z as i32),
    //                 );
                    

    //                 let block = &BLOCKS[id as usize];
    //                 if block.id != 6 {
    //                     set_vertex_standart_block(&mut buffer, x, y, z, block, &chunks);
    //                 }
    //                 // } else {
    //                 //     set_vertex_half_block(&mut buffer, &light, block, chunks.clone(), chunk.chunk_coords, local);
    //                 // }
    //             }
    //         }
    //     }
    //     (buffer.buffer, buffer.index_buffer)
    // }


    pub fn render_test(&mut self, chunk_index: usize, chunks: &mut Chunks) -> (Vec<BlockVertex>, Vec<u16>) {
        let mut buffer = Buffer::new();
        let light_handler = LightHandler::new(&chunks);
        let chunk = chunks.chunks[chunk_index].as_ref().unwrap();


        for ly in 0..CHUNK_SIZE {
            let mut greedy_vertices_top = GreedMesh::new();
            let mut greedy_vertices_bottom = GreedMesh::new();
            for lz in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let id = chunk.voxel((lx, ly, lz)).id;
                    if id == 0 { continue };
                    let (x, y, z) = Chunks::global_coords(chunk.chunk_coords, (lx, ly, lz));

                    let block = &BLOCKS[id as usize];
                    
                    let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
                    let (gmx, gpx) = (x as f32, x as f32+1.0);
                    let (gmy, gpy) = (y as f32, y as f32+1.0);
                    let (gmz, gpz) = (z as f32, z as f32+1.0);
                    if !is_blocked(x, y+1, z, chunks, LightPermeability::UP, block.light_permeability) {
                        let layer = block.faces[3] as f32;

                        let ld = light_handler.light_default((x, py, z));
                        let l0 = light_handler.light_numbered(&ld, (nx,py,z), (nx,py,nz), (x,py,nz));
                        let l1 = light_handler.light_numbered(&ld, (nx,py,z), (nx,py,pz), (x,py,pz));
                        let l2 = light_handler.light_numbered(&ld, (px,py,z), (px,py,pz), (x,py,pz));
                        let l3 = light_handler.light_numbered(&ld, (px,py,z), (px,py,nz), (x,py,nz));

                        greedy_vertices_top.push(lz, lx, [
                            get_block_vertex(gmx, gpy, gmz, 0., 0., layer, &l0),
                            get_block_vertex(gmx, gpy, gpz, 0., 1., layer, &l1),
                            get_block_vertex(gpx, gpy, gpz, 1., 1., layer, &l2),
                            get_block_vertex(gpx, gpy, gmz, 1., 0., layer, &l3)
                        ]);
                    }
                    if !is_blocked(x, y-1, z, chunks, LightPermeability::DOWN, block.light_permeability) {
                        let layer = block.faces[2] as f32;
                
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
            greedy_vertices_top.greed(&mut buffer, &Axis::Y, &[0,1,2,0,2,3]);
            greedy_vertices_bottom.greed(&mut buffer, &Axis::Y, &[3,2,0,2,1,0]);
        }


        for lx in 0..CHUNK_SIZE {
            let mut greedy_vertices_right = GreedMesh::new();
            let mut greedy_vertices_left = GreedMesh::new();
            for ly in 0..CHUNK_SIZE {
                for lz in 0..CHUNK_SIZE {
                    let id = chunk.voxel((lx, ly, lz)).id;
                    if id == 0 { continue };
                    let (x, y, z) = Chunks::global_coords(chunk.chunk_coords, (lx, ly, lz));

                    let block = &BLOCKS[id as usize];
                    
                    let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
                    let (gmx, gpx) = (x as f32, x as f32+1.0);
                    let (gmy, gpy) = (y as f32, y as f32+1.0);
                    let (gmz, gpz) = (z as f32, z as f32+1.0);
                    if !is_blocked(x+1, y, z, chunks, LightPermeability::RIGHT, block.light_permeability) {
                        let layer = block.faces[3] as f32;

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
                    if !is_blocked(x-1, y, z, chunks, LightPermeability::LEFT, block.light_permeability) {
                        let layer = block.faces[2] as f32;
                
                        let ld = light_handler.light_default((nx, y, z));
                        let l0 = light_handler.light_numbered(&ld, (nx,ny,nz), (nx,y,nz), (nx,ny,z));
                        let l1 = light_handler.light_numbered(&ld, (nx,py,pz), (nx,y,pz), (nx,py,z));
                        let l2 = light_handler.light_numbered(&ld, (nx,py,nz), (nx,y,nz), (nx,py,z));
                        let l3 = light_handler.light_numbered(&ld, (nx,ny,pz), (nx,y,pz), (nx,ny,z));
                

                        greedy_vertices_left.push(ly, lz, [
                            get_block_vertex(gmx, gmy, gmz, 0., 0., layer, &l0),
                            get_block_vertex(gmx, gpy, gmz, 0., 1., layer, &l2),
                            get_block_vertex(gmx, gpy, gpz, 1., 1., layer, &l1),
                            get_block_vertex(gmx, gmy, gpz, 1., 0., layer, &l3)
                        ]);
                    }
                }
                
            }
            greedy_vertices_right.greed(&mut buffer, &Axis::X, &[0,1,2,0,2,3]);
            greedy_vertices_left.greed(&mut buffer, &Axis::X, &[3,2,0,2,1,0]);
        }


        for lz in 0..CHUNK_SIZE {
            let mut greedy_vertices_front = GreedMesh::new();
            let mut greedy_vertices_back = GreedMesh::new();
            for lx in 0..CHUNK_SIZE {
                for ly in 0..CHUNK_SIZE {
                    let id = chunk.voxel((lx, ly, lz)).id;
                    if id == 0 { continue };
                    let (x, y, z) = Chunks::global_coords(chunk.chunk_coords, (lx, ly, lz));

                    let block = &BLOCKS[id as usize];
                    
                    let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
                    let (gmx, gpx) = (x as f32, x as f32+1.0);
                    let (gmy, gpy) = (y as f32, y as f32+1.0);
                    let (gmz, gpz) = (z as f32, z as f32+1.0);
                    if !is_blocked(x, y, z+1, chunks, LightPermeability::FRONT, block.light_permeability) {
                        let layer = block.faces[3] as f32;

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
                    if !is_blocked(x, y, z-1, chunks, LightPermeability::BACK, block.light_permeability) {
                        let layer = block.faces[2] as f32;
                
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
            greedy_vertices_front.greed(&mut buffer, &Axis::Z, &[0,1,2,0,2,3]);
            greedy_vertices_back.greed(&mut buffer, &Axis::Z, &[3,2,0,2,1,0]);
        }
        (buffer.buffer, buffer.index_buffer)
    }
}

// fn set_vertex_standart_block(buffer: &mut Buffer, x: i32, y: i32, z: i32, block: &Block, chunks: &Chunks) {
//     let light_handler = LightHandler::new(chunks);
//     let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
//     let (gmx, gpx) = (x as f32, x as f32+1.0);
//     let (gmy, gpy) = (y as f32, y as f32+1.0);
//     let (gmz, gpz) = (z as f32, z as f32+1.0);
//     if !is_blocked(x, y+1, z, chunks.clone(), LightPermeability::UP, block.light_permeability) {
//         let layer = block.faces[3] as f32;

//         let ld = light_handler.light_default((x, py, z));
//         let l0 = light_handler.light_numbered(&ld, (nx,py,z), (nx,py,nz), (x,py,nz));
//         let l1 = light_handler.light_numbered(&ld, (nx,py,z), (nx,py,pz), (x,py,pz));
//         let l2 = light_handler.light_numbered(&ld, (px,py,z), (px,py,pz), (x,py,pz));
//         let l3 = light_handler.light_numbered(&ld, (px,py,z), (px,py,nz), (x,py,nz));

//         let index0 = buffer.push_vertex(gmx, gpy, gmz, 0., 0., layer, &l0);
//         buffer.push_vertex(gmx, gpy, gpz, 0., 1., layer, &l1);
//         let index2 = buffer.push_vertex(gpx, gpy, gpz, 1., 1., layer, &l2);

//         buffer.push_index(index0);
//         buffer.push_index(index2);
//         buffer.push_vertex(gpx, gpy, gmz, 1., 0., layer, &l3);
//     }
    
//     if !is_blocked(x, y-1, z, chunks.clone(), LightPermeability::DOWN, block.light_permeability) {
//         let layer = block.faces[2] as f32;

//         let ld = light_handler.light_default((x, ny, z));
//         let l0 = light_handler.light_numbered(&ld, (nx,ny,nz), (nx,ny,z), (x,ny,nz));
//         let l1 = light_handler.light_numbered(&ld, (px,ny,pz), (px,ny,z), (x,ny,pz));
//         let l2 = light_handler.light_numbered(&ld, (nx,ny,pz), (nx,ny,z), (x,ny,pz));
//         let l3 = light_handler.light_numbered(&ld, (px,ny,nz), (px,ny,z), (x,ny,nz));

//         let index0 = buffer.push_vertex(gmx, gmy, gmz, 0., 0., layer, &l0);
//         let index1 = buffer.push_vertex(gpx, gmy, gpz, 1., 1., layer, &l1);
//         buffer.push_vertex(gmx, gmy, gpz, 0., 1., layer, &l2);

//         buffer.push_index(index0);
//         buffer.push_vertex(gpx, gmy, gmz, 1., 0., layer, &l3);
//         buffer.push_index(index1);
//     }
    
//     if !is_blocked(x+1, y, z, chunks.clone(), LightPermeability::RIGHT, block.light_permeability) {
//         let layer = block.faces[1] as f32;

//         let ld = light_handler.light_default((px, y, z));
//         let l0 = light_handler.light_numbered(&ld, (px,ny,nz), (px,y,nz), (px,ny,z));
//         let l1 = light_handler.light_numbered(&ld, (px,py,nz), (px,y,nz), (px,py,z));
//         let l2 = light_handler.light_numbered(&ld, (px,py,pz), (px,y,pz), (px,py,z));
//         let l3 = light_handler.light_numbered(&ld, (px,ny,pz), (px,y,pz), (px,ny,z));

//         let index0 = buffer.push_vertex(gpx, gmy, gmz, 0., 0., layer, &l0);
//         buffer.push_vertex(gpx, gpy, gmz, 1., 0., layer, &l1);
//         let index2 = buffer.push_vertex(gpx, gpy, gpz, 1., 1., layer, &l2);

//         buffer.push_index(index0);
//         buffer.push_index(index2);
//         buffer.push_vertex(gpx, gmy, gpz, 0., 1., layer, &l3);
//     }
//     if !is_blocked(x-1, y, z, chunks.clone(), LightPermeability::LEFT, block.light_permeability) {
//         let layer = block.faces[0] as f32;

//         let ld = light_handler.light_default((nx, y, z));
//         let l0 = light_handler.light_numbered(&ld, (nx,ny,nz), (nx,y,nz), (nx,ny,z));
//         let l1 = light_handler.light_numbered(&ld, (nx,py,pz), (nx,y,pz), (nx,py,z));
//         let l2 = light_handler.light_numbered(&ld, (nx,py,nz), (nx,y,nz), (nx,py,z));
//         let l3 = light_handler.light_numbered(&ld, (nx,ny,pz), (nx,y,pz), (nx,ny,z));

//         let index0 = buffer.push_vertex(gmx, gmy, gmz, 0., 0., layer, &l0);
//         let index1 = buffer.push_vertex(gmx, gpy, gpz, 1., 1., layer, &l1);
//         buffer.push_vertex(gmx, gpy, gmz, 1., 0., layer, &l2);

//         buffer.push_index(index0);
//         buffer.push_vertex(gmx, gmy, gpz, 0., 1., layer, &l3);
//         buffer.push_index(index1);
//     }
//     if !is_blocked(x, y, z+1, chunks.clone(), LightPermeability::FRONT, block.light_permeability) {
//         let layer = block.faces[5] as f32;

//         let ld = light_handler.light_default((x, y, pz));
//         let l0 = light_handler.light_numbered(&ld, (nx,ny,pz), (x,ny,pz), (nx,y,pz));
//         let l1 = light_handler.light_numbered(&ld, (px,py,pz), (x,py,pz), (px,y,pz));
//         let l2 = light_handler.light_numbered(&ld, (nx,py,pz), (x,py,pz), (nx,y,pz));
//         let l3 = light_handler.light_numbered(&ld, (px,ny,pz), (x,ny,pz), (px,y,pz));

//         let index0 = buffer.push_vertex(gmx, gmy, gpz, 0., 0., layer, &l0);
//         let index1 = buffer.push_vertex(gpx, gpy, gpz, 1., 1., layer, &l1);
//         buffer.push_vertex(gmx, gpy, gpz, 0., 1., layer, &l2);

//         buffer.push_index(index0);
//         buffer.push_vertex(gpx, gmy, gpz, 1., 0., layer, &l3);
//         buffer.push_index(index1);
//     }
//     if !is_blocked(x, y, z-1, chunks.clone(), LightPermeability::BACK, block.light_permeability) {
//         let layer = block.faces[4] as f32;

//         let ld = light_handler.light_default((x, y, nz));
//         let l0 = light_handler.light_numbered(&ld, (nx,ny,nz), (x,ny,nz), (nx,y,nz));
//         let l1 = light_handler.light_numbered(&ld, (nx,py,nz), (x,py,nz), (nx,y,nz));
//         let l2 = light_handler.light_numbered(&ld, (px,py,nz), (x,py,nz), (px,y,nz));
//         let l3 = light_handler.light_numbered(&ld, (px,ny,nz), (x,ny,nz), (px,y,nz));

//         let index0 = buffer.push_vertex(gmx, gmy, gmz, 0., 0., layer, &l0);
//         buffer.push_vertex(gmx, gpy, gmz, 0., 1., layer, &l1);
//         let index2 = buffer.push_vertex(gpx, gpy, gmz, 1., 1., layer, &l2);

//         buffer.push_index(index0);
//         buffer.push_index(index2);
//         buffer.push_vertex(gpx, gmy, gmz, 1., 0., layer, &l3);
//     }
// }

// fn set_vertex_half_block(buffer: &mut Buffer, l: &Light, block: &Block, chunks: Arc<&mut Chunks>, chunk_xyz: (i32, i32, i32), local: (i32, i32, i32)) {
//     let (global_x, global_y, global_z) = (
//         x as f32 + chunk_xyz.0 as f32*CHUNK_WIDTH as f32,
//         y as f32 + chunk_xyz.1 as f32*CHUNK_HEIGHT as f32,
//         z as f32 + chunk_xyz.2 as f32*CHUNK_DEPTH as f32);
    
//     let (gmx, gpx) = (global_x, global_x+1.0);
//     let (gmy, gpy) = (global_y, global_y+0.5);
//     let (gmz, gpz) = (global_z, global_z+1.0);
//     if !is_blocked(chunk_xyz, (x, y+1, z), chunks.clone(), LightPermeability::UP, block.light_permeability) {
//         let top_normal = [0.0, 1.0, 0.0];
//         let layer = block.faces[3] as f32;

//         let lr = l.get(x, y+1, z, 0)/15.0;
//         let lg = l.get(x, y+1, z, 1)/15.0;
//         let lb = l.get(x, y+1, z, 2)/15.0;
//         let ls = l.get(x, y+1, z, 3)/15.0;

//         let lr0 = (l.get(x-1,y+1,z,0) + lr*30.0 + l.get(x-1,y+1,z-1,0) + l.get(x,y+1,z-1,0)) / 5.0 / 15.0;
//         let lr1 = (l.get(x-1,y+1,z,0) + lr*30.0 + l.get(x-1,y+1,z+1,0) + l.get(x,y+1,z+1,0)) / 5.0 / 15.0;
//         let lr2 = (l.get(x+1,y+1,z,0) + lr*30.0 + l.get(x+1,y+1,z+1,0) + l.get(x,y+1,z+1,0)) / 5.0 / 15.0;
//         let lr3 = (l.get(x+1,y+1,z,0) + lr*30.0 + l.get(x+1,y+1,z-1,0) + l.get(x,y+1,z-1,0)) / 5.0 / 15.0;

//         let lg0 = (l.get(x-1,y+1,z,1) + lg*30.0 + l.get(x-1,y+1,z-1,1) + l.get(x,y+1,z-1,1)) / 5.0 / 15.0;
//         let lg1 = (l.get(x-1,y+1,z,1) + lg*30.0 + l.get(x-1,y+1,z+1,1) + l.get(x,y+1,z+1,1)) / 5.0 / 15.0;
//         let lg2 = (l.get(x+1,y+1,z,1) + lg*30.0 + l.get(x+1,y+1,z+1,1) + l.get(x,y+1,z+1,1)) / 5.0 / 15.0;
//         let lg3 = (l.get(x+1,y+1,z,1) + lg*30.0 + l.get(x+1,y+1,z-1,1) + l.get(x,y+1,z-1,1)) / 5.0 / 15.0;

//         let lb0 = (l.get(x-1,y+1,z,2) + lb*30.0 + l.get(x-1,y+1,z-1,2) + l.get(x,y+1,z-1,2)) / 5.0 / 15.0;
//         let lb1 = (l.get(x-1,y+1,z,2) + lb*30.0 + l.get(x-1,y+1,z+1,2) + l.get(x,y+1,z+1,2)) / 5.0 / 15.0;
//         let lb2 = (l.get(x+1,y+1,z,2) + lb*30.0 + l.get(x+1,y+1,z+1,2) + l.get(x,y+1,z+1,2)) / 5.0 / 15.0;
//         let lb3 = (l.get(x+1,y+1,z,2) + lb*30.0 + l.get(x+1,y+1,z-1,2) + l.get(x,y+1,z-1,2)) / 5.0 / 15.0;

//         let ls0 = (l.get(x-1,y+1,z,3) + ls*30.0 + l.get(x-1,y+1,z-1,3) + l.get(x,y+1,z-1,3)) / 5.0 / 15.0;
//         let ls1 = (l.get(x-1,y+1,z,3) + ls*30.0 + l.get(x-1,y+1,z+1,3) + l.get(x,y+1,z+1,3)) / 5.0 / 15.0;
//         let ls2 = (l.get(x+1,y+1,z,3) + ls*30.0 + l.get(x+1,y+1,z+1,3) + l.get(x,y+1,z+1,3)) / 5.0 / 15.0;
//         let ls3 = (l.get(x+1,y+1,z,3) + ls*30.0 + l.get(x+1,y+1,z-1,3) + l.get(x,y+1,z-1,3)) / 5.0 / 15.0;

//         buffer.vertex(gmx, gpy, gmz, 0., 0., layer, top_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gmx, gpy, gpz, 0., 1., layer, top_normal, lr1, lg1, lb1, ls1);
//         buffer.vertex(gpx, gpy, gpz, 1., 1., layer, top_normal, lr2, lg2, lb2, ls2);

//         buffer.vertex(gmx, gpy, gmz, 0., 0., layer, top_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gpy, gpz, 1., 1., layer, top_normal, lr2, lg2, lb2, ls2);
//         buffer.vertex(gpx, gpy, gmz, 1., 0., layer, top_normal, lr3, lg3, lb3, ls3);
//     }
        
    
//     if !is_blocked(chunk_xyz, (x, y-1, z), chunks.clone(), LightPermeability::DOWN, block.light_permeability) {
//         let bottom_normal = [0.0, -1.0, 0.0];
//         let layer = block.faces[2] as f32;

//         let lr = l.get(x,y-1,z, 0) / 15.0;
//         let lg = l.get(x,y-1,z, 1) / 15.0;
//         let lb = l.get(x,y-1,z, 2) / 15.0;
//         let ls = l.get(x,y-1,z, 3) / 15.0;

//         let lr0 = (l.get(x-1,y-1,z-1,0) + lr*30.0 + l.get(x-1,y-1,z,0) + l.get(x,y-1,z-1,0)) / 5.0 / 15.0;
//         let lr1 = (l.get(x+1,y-1,z+1,0) + lr*30.0 + l.get(x+1,y-1,z,0) + l.get(x,y-1,z+1,0)) / 5.0 / 15.0;
//         let lr2 = (l.get(x-1,y-1,z+1,0) + lr*30.0 + l.get(x-1,y-1,z,0) + l.get(x,y-1,z+1,0)) / 5.0 / 15.0;
//         let lr3 = (l.get(x+1,y-1,z-1,0) + lr*30.0 + l.get(x+1,y-1,z,0) + l.get(x,y-1,z-1,0)) / 5.0 / 15.0;

//         let lg0 = (l.get(x-1,y-1,z-1,1) + lg*30.0 + l.get(x-1,y-1,z,1) + l.get(x,y-1,z-1,1)) / 5.0 / 15.0;
//         let lg1 = (l.get(x+1,y-1,z+1,1) + lg*30.0 + l.get(x+1,y-1,z,1) + l.get(x,y-1,z+1,1)) / 5.0 / 15.0;
//         let lg2 = (l.get(x-1,y-1,z+1,1) + lg*30.0 + l.get(x-1,y-1,z,1) + l.get(x,y-1,z+1,1)) / 5.0 / 15.0;
//         let lg3 = (l.get(x+1,y-1,z-1,1) + lg*30.0 + l.get(x+1,y-1,z,1) + l.get(x,y-1,z-1,1)) / 5.0 / 15.0;

//         let lb0 = (l.get(x-1,y-1,z-1,2) + lb*30.0 + l.get(x-1,y-1,z,2) + l.get(x,y-1,z-1,2)) / 5.0 / 15.0;
//         let lb1 = (l.get(x+1,y-1,z+1,2) + lb*30.0 + l.get(x+1,y-1,z,2) + l.get(x,y-1,z+1,2)) / 5.0 / 15.0;
//         let lb2 = (l.get(x-1,y-1,z+1,2) + lb*30.0 + l.get(x-1,y-1,z,2) + l.get(x,y-1,z+1,2)) / 5.0 / 15.0;
//         let lb3 = (l.get(x+1,y-1,z-1,2) + lb*30.0 + l.get(x+1,y-1,z,2) + l.get(x,y-1,z-1,2)) / 5.0 / 15.0;

//         let ls0 = (l.get(x-1,y-1,z-1,3) + ls*30.0 + l.get(x-1,y-1,z,3) + l.get(x,y-1,z-1,3)) / 5.0 / 15.0;
//         let ls1 = (l.get(x+1,y-1,z+1,3) + ls*30.0 + l.get(x+1,y-1,z,3) + l.get(x,y-1,z+1,3)) / 5.0 / 15.0;
//         let ls2 = (l.get(x-1,y-1,z+1,3) + ls*30.0 + l.get(x-1,y-1,z,3) + l.get(x,y-1,z+1,3)) / 5.0 / 15.0;
//         let ls3 = (l.get(x+1,y-1,z-1,3) + ls*30.0 + l.get(x+1,y-1,z,3) + l.get(x,y-1,z-1,3)) / 5.0 / 15.0;

//         buffer.vertex(gmx, gmy, gmz, 0., 0., layer, bottom_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gmy, gpz, 1., 1., layer, bottom_normal, lr1, lg1, lb1, ls1);
//         buffer.vertex(gmx, gmy, gpz, 0., 1., layer, bottom_normal, lr2,lg2,lb2,ls2);

//         buffer.vertex(gmx, gmy, gmz, 0., 0., layer, bottom_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gmy, gmz, 1., 0., layer, bottom_normal, lr3, lg3, lb3, ls3);
//         buffer.vertex(gpx, gmy, gpz, 1., 1., layer, bottom_normal, lr1, lg1, lb1, ls1);
//     }

    
//     if !is_blocked(chunk_xyz, (x+1, y, z), chunks.clone(), LightPermeability::RIGHT, block.light_permeability) {
//         let right_normal = [1.0, 0.0, 0.0];
//         let layer = block.faces[1] as f32;

//         let lr = l.get(x+1,y,z, 0) / 15.0;
//         let lg = l.get(x+1,y,z, 1) / 15.0;
//         let lb = l.get(x+1,y,z, 2) / 15.0;
//         let ls = l.get(x+1,y,z, 3) / 15.0;

//         let lr0 = (l.get(x+1,y-1,z-1,0) + lr*30.0 + l.get(x+1,y,z-1,0) + l.get(x+1,y-1,z,0)) / 5.0 / 15.0;
//         let lr1 = (l.get(x+1,y+1,z-1,0) + lr*30.0 + l.get(x+1,y,z-1,0) + l.get(x+1,y+1,z,0)) / 5.0 / 15.0;
//         let lr2 = (l.get(x+1,y+1,z+1,0) + lr*30.0 + l.get(x+1,y,z+1,0) + l.get(x+1,y+1,z,0)) / 5.0 / 15.0;
//         let lr3 = (l.get(x+1,y-1,z+1,0) + lr*30.0 + l.get(x+1,y,z+1,0) + l.get(x+1,y-1,z,0)) / 5.0 / 15.0;

//         let lg0 = (l.get(x+1,y-1,z-1,1) + lg*30.0 + l.get(x+1,y,z-1,1) + l.get(x+1,y-1,z,1)) / 5.0 / 15.0;
//         let lg1 = (l.get(x+1,y+1,z-1,1) + lg*30.0 + l.get(x+1,y,z-1,1) + l.get(x+1,y+1,z,1)) / 5.0 / 15.0;
//         let lg2 = (l.get(x+1,y+1,z+1,1) + lg*30.0 + l.get(x+1,y,z+1,1) + l.get(x+1,y+1,z,1)) / 5.0 / 15.0;
//         let lg3 = (l.get(x+1,y-1,z+1,1) + lg*30.0 + l.get(x+1,y,z+1,1) + l.get(x+1,y-1,z,1)) / 5.0 / 15.0;

//         let lb0 = (l.get(x+1,y-1,z-1,2) + lb*30.0 + l.get(x+1,y,z-1,2) + l.get(x+1,y-1,z,2)) / 5.0 / 15.0;
//         let lb1 = (l.get(x+1,y+1,z-1,2) + lb*30.0 + l.get(x+1,y,z-1,2) + l.get(x+1,y+1,z,2)) / 5.0 / 15.0;
//         let lb2 = (l.get(x+1,y+1,z+1,2) + lb*30.0 + l.get(x+1,y,z+1,2) + l.get(x+1,y+1,z,2)) / 5.0 / 15.0;
//         let lb3 = (l.get(x+1,y-1,z+1,2) + lb*30.0 + l.get(x+1,y,z+1,2) + l.get(x+1,y-1,z,2)) / 5.0 / 15.0;

//         let ls0 = (l.get(x+1,y-1,z-1,3) + ls*30.0 + l.get(x+1,y,z-1,3) + l.get(x+1,y-1,z,3)) / 5.0 / 15.0;
//         let ls1 = (l.get(x+1,y+1,z-1,3) + ls*30.0 + l.get(x+1,y,z-1,3) + l.get(x+1,y+1,z,3)) / 5.0 / 15.0;
//         let ls2 = (l.get(x+1,y+1,z+1,3) + ls*30.0 + l.get(x+1,y,z+1,3) + l.get(x+1,y+1,z,3)) / 5.0 / 15.0;
//         let ls3 = (l.get(x+1,y-1,z+1,3) + ls*30.0 + l.get(x+1,y,z+1,3) + l.get(x+1,y-1,z,3)) / 5.0 / 15.0;

//         buffer.vertex(gpx, gmy, gmz, 0., 0., layer, right_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gpy, gmz, 0.5, 0., layer, right_normal, lr1, lg1, lb1, ls1);
//         buffer.vertex(gpx, gpy, gpz, 0.5, 1., layer, right_normal, lr2,lg2,lb2,ls2);

//         buffer.vertex(gpx, gmy, gmz, 0., 0., layer, right_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gpy, gpz, 0.5, 1., layer, right_normal, lr2,lg2,lb2,ls2);
//         buffer.vertex(gpx, gmy, gpz, 0., 1., layer, right_normal, lr3, lg3, lb3, ls3);
//     }
//     if !is_blocked(chunk_xyz, (x-1, y, z), chunks.clone(), LightPermeability::LEFT, block.light_permeability) {
//         let left_normal = [-1.0, 0.0, 0.0];
//         let layer = block.faces[0] as f32;

//         let lr = l.get(x-1,y,z, 0) / 15.0;
//         let lg = l.get(x-1,y,z, 1) / 15.0;
//         let lb = l.get(x-1,y,z, 2) / 15.0;
//         let ls = l.get(x-1,y,z, 3) / 15.0;

//         let lr0 = (l.get(x-1,y-1,z-1,0) + lr*30.0 + l.get(x-1,y,z-1,0) + l.get(x-1,y-1,z,0)) / 5.0 / 15.0;
//         let lr1 = (l.get(x-1,y+1,z+1,0) + lr*30.0 + l.get(x-1,y,z+1,0) + l.get(x-1,y+1,z,0)) / 5.0 / 15.0;
//         let lr2 = (l.get(x-1,y+1,z-1,0) + lr*30.0 + l.get(x-1,y,z-1,0) + l.get(x-1,y+1,z,0)) / 5.0 / 15.0;
//         let lr3 = (l.get(x-1,y-1,z+1,0) + lr*30.0 + l.get(x-1,y,z+1,0) + l.get(x-1,y-1,z,0)) / 5.0 / 15.0;

//         let lg0 = (l.get(x-1,y-1,z-1,1) + lg*30.0 + l.get(x-1,y,z-1,1) + l.get(x-1,y-1,z,1)) / 5.0 / 15.0;
//         let lg1 = (l.get(x-1,y+1,z+1,1) + lg*30.0 + l.get(x-1,y,z+1,1) + l.get(x-1,y+1,z,1)) / 5.0 / 15.0;
//         let lg2 = (l.get(x-1,y+1,z-1,1) + lg*30.0 + l.get(x-1,y,z-1,1) + l.get(x-1,y+1,z,1)) / 5.0 / 15.0;
//         let lg3 = (l.get(x-1,y-1,z+1,1) + lg*30.0 + l.get(x-1,y,z+1,1) + l.get(x-1,y-1,z,1)) / 5.0 / 15.0;

//         let lb0 = (l.get(x-1,y-1,z-1,2) + lb*30.0 + l.get(x-1,y,z-1,2) + l.get(x-1,y-1,z,2)) / 5.0 / 15.0;
//         let lb1 = (l.get(x-1,y+1,z+1,2) + lb*30.0 + l.get(x-1,y,z+1,2) + l.get(x-1,y+1,z,2)) / 5.0 / 15.0;
//         let lb2 = (l.get(x-1,y+1,z-1,2) + lb*30.0 + l.get(x-1,y,z-1,2) + l.get(x-1,y+1,z,2)) / 5.0 / 15.0;
//         let lb3 = (l.get(x-1,y-1,z+1,2) + lb*30.0 + l.get(x-1,y,z+1,2) + l.get(x-1,y-1,z,2)) / 5.0 / 15.0;

//         let ls0 = (l.get(x-1,y-1,z-1,3) + ls*30.0 + l.get(x-1,y,z-1,3) + l.get(x-1,y-1,z,3)) / 5.0 / 15.0;
//         let ls1 = (l.get(x-1,y+1,z+1,3) + ls*30.0 + l.get(x-1,y,z+1,3) + l.get(x-1,y+1,z,3)) / 5.0 / 15.0;
//         let ls2 = (l.get(x-1,y+1,z-1,3) + ls*30.0 + l.get(x-1,y,z-1,3) + l.get(x-1,y+1,z,3)) / 5.0 / 15.0;
//         let ls3 = (l.get(x-1,y-1,z+1,3) + ls*30.0 + l.get(x-1,y,z+1,3) + l.get(x-1,y-1,z,3)) / 5.0 / 15.0;

//         buffer.vertex(gmx, gmy, gmz, 0., 0., layer, left_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gmx, gpy, gpz, 0.5, 1., layer, left_normal, lr1, lg1, lb1, ls1);
//         buffer.vertex(gmx, gpy, gmz, 0.5, 0., layer, left_normal, lr2,lg2,lb2,ls2);

//         buffer.vertex(gmx, gmy, gmz, 0., 0., layer, left_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gmx, gmy, gpz, 0., 1., layer, left_normal, lr3, lg3, lb3, ls3);
//         buffer.vertex(gmx, gpy, gpz, 0.5, 1., layer, left_normal, lr1, lg1, lb1, ls1);
//     }
//     if !is_blocked(chunk_xyz, (x, y, z+1), chunks.clone(), LightPermeability::FRONT, block.light_permeability) {
//         let front_normal = [0.0, 0.0, 1.0];
//         let layer = block.faces[5] as f32;

//         let lr = l.get(x,y,z+1, 0) / 15.0;
//         let lg = l.get(x,y,z+1, 1) / 15.0;
//         let lb = l.get(x,y,z+1, 2) / 15.0;
//         let ls = l.get(x,y,z+1, 3) / 15.0;

//         let lr0 = (l.get(x-1,y-1,z+1,0) + lr*30.0 + l.get(x,y-1,z+1,0) + l.get(x-1,y,z+1,0)) / 5.0 / 15.0;
//         let lr1 = (l.get(x+1,y+1,z+1,0) + lr*30.0 + l.get(x,y+1,z+1,0) + l.get(x+1,y,z+1,0)) / 5.0 / 15.0;
//         let lr2 = (l.get(x-1,y+1,z+1,0) + lr*30.0 + l.get(x,y+1,z+1,0) + l.get(x-1,y,z+1,0)) / 5.0 / 15.0;
//         let lr3 = (l.get(x+1,y-1,z+1,0) + lr*30.0 + l.get(x,y-1,z+1,0) + l.get(x+1,y,z+1,0)) / 5.0 / 15.0;

//         let lg0 = (l.get(x-1,y-1,z+1,1) + lg*30.0 + l.get(x,y-1,z+1,1) + l.get(x-1,y,z+1,1)) / 5.0 / 15.0;
//         let lg1 = (l.get(x+1,y+1,z+1,1) + lg*30.0 + l.get(x,y+1,z+1,1) + l.get(x+1,y,z+1,1)) / 5.0 / 15.0;
//         let lg2 = (l.get(x-1,y+1,z+1,1) + lg*30.0 + l.get(x,y+1,z+1,1) + l.get(x-1,y,z+1,1)) / 5.0 / 15.0;
//         let lg3 = (l.get(x+1,y-1,z+1,1) + lg*30.0 + l.get(x,y-1,z+1,1) + l.get(x+1,y,z+1,1)) / 5.0 / 15.0;

//         let lb0 = (l.get(x-1,y-1,z+1,2) + lb*30.0 + l.get(x,y-1,z+1,2) + l.get(x-1,y,z+1,2)) / 5.0 / 15.0;
//         let lb1 = (l.get(x+1,y+1,z+1,2) + lb*30.0 + l.get(x,y+1,z+1,2) + l.get(x+1,y,z+1,2)) / 5.0 / 15.0;
//         let lb2 = (l.get(x-1,y+1,z+1,2) + lb*30.0 + l.get(x,y+1,z+1,2) + l.get(x-1,y,z+1,2)) / 5.0 / 15.0;
//         let lb3 = (l.get(x+1,y-1,z+1,2) + lb*30.0 + l.get(x,y-1,z+1,2) + l.get(x+1,y,z+1,2)) / 5.0 / 15.0;

//         let ls0 = (l.get(x-1,y-1,z+1,3) + ls*30.0 + l.get(x,y-1,z+1,3) + l.get(x-1,y,z+1,3)) / 5.0 / 15.0;
//         let ls1 = (l.get(x+1,y+1,z+1,3) + ls*30.0 + l.get(x,y+1,z+1,3) + l.get(x+1,y,z+1,3)) / 5.0 / 15.0;
//         let ls2 = (l.get(x-1,y+1,z+1,3) + ls*30.0 + l.get(x,y+1,z+1,3) + l.get(x-1,y,z+1,3)) / 5.0 / 15.0;
//         let ls3 = (l.get(x+1,y-1,z+1,3) + ls*30.0 + l.get(x,y-1,z+1,3) + l.get(x+1,y,z+1,3)) / 5.0 / 15.0;

//         buffer.vertex(gmx, gmy, gpz, 0., 0., layer, front_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gpy, gpz, 1., 0.5, layer, front_normal, lr1, lg1, lb1, ls1);
//         buffer.vertex(gmx, gpy, gpz, 0., 0.5, layer, front_normal, lr2,lg2,lb2,ls2);

//         buffer.vertex(gmx, gmy, gpz, 0., 0., layer, front_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gmy, gpz, 1., 0., layer, front_normal, lr3, lg3, lb3, ls3);
//         buffer.vertex(gpx, gpy, gpz, 1., 0.5, layer, front_normal, lr1, lg1, lb1, ls1);
//     }
//     if !is_blocked(chunk_xyz, (x, y, z-1), chunks.clone(), LightPermeability::BACK, block.light_permeability) {
//         let back_normal = [0.0, 0.0, -1.0];
//         let layer = block.faces[4] as f32;

//         let lr = l.get(x,y,z-1, 0) / 15.0;
//         let lg = l.get(x,y,z-1, 1) / 15.0;
//         let lb = l.get(x,y,z-1, 2) / 15.0;
//         let ls = l.get(x,y,z-1, 3) / 15.0;

//         let lr0 = (l.get(x-1,y-1,z-1,0) + lr*30.0 + l.get(x,y-1,z-1,0) + l.get(x-1,y,z-1,0)) / 5.0 / 15.0;
//         let lr1 = (l.get(x-1,y+1,z-1,0) + lr*30.0 + l.get(x,y+1,z-1,0) + l.get(x-1,y,z-1,0)) / 5.0 / 15.0;
//         let lr2 = (l.get(x+1,y+1,z-1,0) + lr*30.0 + l.get(x,y+1,z-1,0) + l.get(x+1,y,z-1,0)) / 5.0 / 15.0;
//         let lr3 = (l.get(x+1,y-1,z-1,0) + lr*30.0 + l.get(x,y-1,z-1,0) + l.get(x+1,y,z-1,0)) / 5.0 / 15.0;

//         let lg0 = (l.get(x-1,y-1,z-1,1) + lg*30.0 + l.get(x,y-1,z-1,1) + l.get(x-1,y,z-1,1)) / 5.0 / 15.0;
//         let lg1 = (l.get(x-1,y+1,z-1,1) + lg*30.0 + l.get(x,y+1,z-1,1) + l.get(x-1,y,z-1,1)) / 5.0 / 15.0;
//         let lg2 = (l.get(x+1,y+1,z-1,1) + lg*30.0 + l.get(x,y+1,z-1,1) + l.get(x+1,y,z-1,1)) / 5.0 / 15.0;
//         let lg3 = (l.get(x+1,y-1,z-1,1) + lg*30.0 + l.get(x,y-1,z-1,1) + l.get(x+1,y,z-1,1)) / 5.0 / 15.0;

//         let lb0 = (l.get(x-1,y-1,z-1,2) + lb*30.0 + l.get(x,y-1,z-1,2) + l.get(x-1,y,z-1,2)) / 5.0 / 15.0;
//         let lb1 = (l.get(x-1,y+1,z-1,2) + lb*30.0 + l.get(x,y+1,z-1,2) + l.get(x-1,y,z-1,2)) / 5.0 / 15.0;
//         let lb2 = (l.get(x+1,y+1,z-1,2) + lb*30.0 + l.get(x,y+1,z-1,2) + l.get(x+1,y,z-1,2)) / 5.0 / 15.0;
//         let lb3 = (l.get(x+1,y-1,z-1,2) + lb*30.0 + l.get(x,y-1,z-1,2) + l.get(x+1,y,z-1,2)) / 5.0 / 15.0;

//         let ls0 = (l.get(x-1,y-1,z-1,3) + ls*30.0 + l.get(x,y-1,z-1,3) + l.get(x-1,y,z-1,3)) / 5.0 / 15.0;
//         let ls1 = (l.get(x-1,y+1,z-1,3) + ls*30.0 + l.get(x,y+1,z-1,3) + l.get(x-1,y,z-1,3)) / 5.0 / 15.0;
//         let ls2 = (l.get(x+1,y+1,z-1,3) + ls*30.0 + l.get(x,y+1,z-1,3) + l.get(x+1,y,z-1,3)) / 5.0 / 15.0;
//         let ls3 = (l.get(x+1,y-1,z-1,3) + ls*30.0 + l.get(x,y-1,z-1,3) + l.get(x+1,y,z-1,3)) / 5.0 / 15.0;

//         buffer.vertex(gmx, gmy, gmz, 0., 0., layer, back_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gmx, gpy, gmz, 0., 0.5, layer, back_normal, lr1, lg1, lb1, ls1);
//         buffer.vertex(gpx, gpy, gmz, 1., 0.5, layer, back_normal, lr2,lg2,lb2,ls2);

//         buffer.vertex(gmx, gmy, gmz, 0., 0., layer, back_normal, lr0, lg0, lb0, ls0);
//         buffer.vertex(gpx, gpy, gmz, 1., 0.5, layer, back_normal, lr2,lg2,lb2,ls2);
//         buffer.vertex(gpx, gmy, gmz, 1., 0., layer, back_normal, lr3, lg3, lb3, ls3);
//     }
// }

// fn is_blocked(chunk_xyz: (i32, i32, i32), xyz: (i32, i32, i32), chunks: Arc<&mut Chunks>, side: LightPermeability, current: LightPermeability) -> bool {
//     let (mut x, mut y, mut z) = xyz;
//     let (mut chunk_x, mut chunk_y, mut chunk_z) = chunk_xyz;

//     if x >= CHUNK_WIDTH as i32 { chunk_x += 1; x = 0; }
//     if x < 0 { chunk_x -= 1; x = 15; }
//     if y >= CHUNK_HEIGHT as i32 { chunk_y += 1; y = 0; }
//     if y < 0 { chunk_y -= 1; y = 15; }
//     if z >= CHUNK_DEPTH as i32 { chunk_z += 1; z = 0; }
//     if z < 0 { chunk_z -= 1; z = 15; }

//     if chunk_x < 0 || chunk_y < 0 || chunk_z < 0 { return false; }
//     if chunk_x >= chunks.width || chunk_y >= chunks.height || chunk_z >= chunks.depth { return false; }

//     let index = ((chunk_y*chunks.depth+chunk_z)*chunks.width+chunk_x) as usize;
//     if chunks.chunks.get(index).is_none() { return false; }
//     if chunks.chunks.get(index).unwrap().is_none() { return false; }
    
//     let voxel = chunks.get_voxel(chunk_x, chunk_y, chunk_z, x, y, z);
//     if voxel.is_none() { return false; }
//     let block = &BLOCKS[voxel.unwrap().id as usize];

//     ((block.light_permeability & side.get_opposite_side()).bits() == 0) && ((current & side).bits() == 0)
// }


// struct Light<'a> {
//     chunk_xyz: (i32, i32, i32),
//     chunks: Arc<&'a mut Chunks>,
// }
// impl Light<'_> {
//     pub fn new(chunk_xyz: (i32, i32, i32), chunks: Arc<&mut Chunks>) -> Light {
//         Light { chunk_xyz, chunks }
//     }
//     pub fn get(&self, x: i32, y: i32, z: i32, channel: u8) -> f32 {
//         let (mut x, mut y, mut z) = (x, y, z);
//         let (mut chunk_x, mut chunk_y, mut chunk_z) = self.chunk_xyz;

//         if x >= CHUNK_SIZE as i32 { chunk_x += 1; x = 0; }
//         if x < 0 { chunk_x -= 1; x = 15; }
//         if y >= CHUNK_SIZE as i32 { chunk_y += 1; y = 0; }
//         if y < 0 { chunk_y -= 1; y = 15; }
//         if z >= CHUNK_SIZE as i32 { chunk_z += 1; z = 0; }
//         if z < 0 { chunk_z -= 1; z = 15; }

//         if chunk_x < 0 || chunk_y < 0 || chunk_z < 0 { return 0.0; }
//         if chunk_x >= self.chunks.width || chunk_y >= self.chunks.height || chunk_z >= self.chunks.depth { return 0.0; }

//         let index = ((chunk_y*self.chunks.depth+chunk_z)*self.chunks.width+chunk_x) as usize;
//         if self.chunks.chunks.get(index).is_none() { return 0.0; }
//         if self.chunks.chunks.get(index).unwrap().is_none() { return 0.0; }

//         self.chunks.chunks[index].as_ref().unwrap().lightmap.get(x as u8, y as u8, z as u8, channel) as f32
//     }
// }

struct LightHandler<'a> {
    chunks: &'a Chunks,
}

impl<'a> LightHandler<'a> {
    const COEFFICIENT: f32 = 30.0;
    pub fn new(chunks: &'a Chunks) -> Self { Self { chunks }}

    pub fn light_default(&self, face: (i32, i32, i32)) -> Vec<f32> {
        (0..4).map(|item| {
            self.chunks.light(face.0,face.1,face.2,item) as f32*Self::COEFFICIENT/15.0
        }).collect::<Vec<f32>>()
    }

    pub fn light_numbered(&self, light_default: &Vec<f32>, c1: (i32, i32, i32), c2: (i32, i32, i32), c3: (i32, i32, i32)) -> Vec<f32> {
        light_default.iter().enumerate().map(|(i, light)| {
            (self.chunks.light(c1.0,c1.1,c1.2,i as u8) as f32 +
             self.chunks.light(c2.0,c2.1,c2.2,i as u8) as f32 +
             self.chunks.light(c3.0,c3.1,c3.2,i as u8) as f32 +
             light) / 75.0
        }).collect::<Vec<f32>>()
    }
}

fn is_blocked(x: i32, y: i32, z: i32, chunks: &Chunks, side: LightPermeability, current: LightPermeability) -> bool {
    if let Some(voxel) = chunks.voxel_global(x, y, z) {
        let block = &BLOCKS[voxel.id as usize];
        return((block.light_permeability & side.get_opposite_side()).bits() == 0) && ((current & side).bits() == 0)
    }
    false
}