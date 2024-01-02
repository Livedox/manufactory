use itertools::iproduct;

use crate::{voxels::{chunks::{Chunks, WORLD_HEIGHT}, chunk::CHUNK_SIZE, block::blocks::BLOCKS}, world::chunk_coords::ChunkCoords};

use super::light_solver::LightSolver;
const MAX_LIGHT: u16 = 15;
const SIDE_COORDS_OFFSET: [(i32, i32, i32); 6] = [
    (1,0,0), (-1,0,0),
    (0,1,0), (0,-1,0),
    (0,0,1), (0,0,-1),
];

#[derive(Debug)]
pub struct LightSolvers {
    solver_red: LightSolver,
    solver_green: LightSolver,
    solver_blue: LightSolver,
    pub solver_sun: LightSolver,
}


impl LightSolvers {
    pub fn new() -> Self {Self {
        solver_red: LightSolver::new(0),
        solver_green: LightSolver::new(1),
        solver_blue: LightSolver::new(2),
        solver_sun: LightSolver::new(3),
    }}


    pub fn build_sky_light_chunk(&mut self, chunks: &mut Chunks, cx: i32, cy: i32, cz: i32) {
        let chunks_ptr = chunks as *mut Chunks;
        let ox = chunks.ox;
        let oz = chunks.oz;
        let cx = cx - ox;
        let cz = cz - oz;
        let Some(chunk) = chunks.mut_local_chunk(ChunkCoords(cx, cy, cz)) else {return};

        if chunk.xyz.1 == (WORLD_HEIGHT-1) as i32 {
            for (lz, lx) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE) {
                chunk.lightmap.set_sun((lx as u8, (CHUNK_SIZE-1) as u8, lz as u8), 15);
            }
        }

        if let Some(top_chunk) = unsafe {chunks_ptr.as_ref().expect("Chunks don't found").local_chunk(ChunkCoords(cx, cy+1, cz))} {
            for (lz, lx) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE) {
                if top_chunk.lightmap.get_sun((lx as u8, 0, lz as u8)) == 15 {
                    chunk.lightmap.set_sun((lx as u8, (CHUNK_SIZE-1) as u8, lz as u8), 15);
                }
            }
        }

        for (ly, lz, lx) in iproduct!((0..CHUNK_SIZE-1).rev(), 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            if chunk.lightmap.get_sun((lx as u8, (ly+1) as u8, lz as u8)) == 15
                && BLOCKS()[chunk.voxel((lx as u8, ly as u8, lz as u8).into()).id as usize].id() == 0 {
                chunk.lightmap.set_sun((lx as u8, ly as u8, lz as u8), 15);
                let global = ChunkCoords(cx + ox, cy, cz + oz)
                    .to_global((lx as u8, ly as u8, lz as u8).into());
                self.solver_sun.add_debug(unsafe {chunks_ptr.as_mut().unwrap()}, global.0, global.1, global.2);
            }
        }
        self.solver_sun.solve(chunks);
    }


    pub fn build_sky_light(&mut self, chunks: &mut Chunks) {
        for (cy, cz, cx) in iproduct!((0..chunks.height).rev(), 0..chunks.depth, 0..chunks.width) {
            let chunks_ptr = chunks as *mut Chunks;
            let Some(chunk) = chunks.mut_local_chunk(ChunkCoords(cx, cy, cz)) else {continue};

            if chunk.xyz.1 == (WORLD_HEIGHT-1) as i32 {
                for (lz, lx) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE) {
                    chunk.lightmap.set_sun((lx as u8, (CHUNK_SIZE-1) as u8, lz as u8), 15);
                }
            }

            if let Some(top_chunk) = unsafe {chunks_ptr.as_ref().expect("Chunks don't found").local_chunk(ChunkCoords(cx, cy+1, cz))} {
                for (lz, lx) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE) {
                    if top_chunk.lightmap.get_sun((lx as u8, 0, lz as u8)) == 15 {
                        chunk.lightmap.set_sun((lx as u8, (CHUNK_SIZE-1) as u8, lz as u8), 15);
                    }
                }
            }

            for (ly, lz, lx) in iproduct!((0..CHUNK_SIZE-1).rev(), 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
                if chunk.lightmap.get_sun((lx as u8, (ly+1) as u8, lz as u8)) == 15
                 && BLOCKS()[chunk.voxel((lx as u8, ly as u8, lz as u8).into()).id as usize].id() == 0 {
                    chunk.lightmap.set_sun((lx as u8, ly as u8, lz as u8), 15);
                    let global = ChunkCoords(cx, cy, cz).to_global((lx as u8, ly as u8, lz as u8).into());
                    self.solver_sun.add(unsafe {chunks_ptr.as_mut().unwrap()}, global.0, global.1, global.2);
                }
            }
        }
        self.solver_sun.solve(chunks);
    }


    pub fn on_chunk_loaded(&mut self, chunks: &mut Chunks, cx: i32, cy: i32, cz: i32) {
        for (ly, lz, lx) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            let xyz = ChunkCoords(cx, cy, cz).to_global((lx as u8, ly as u8, lz as u8).into());
            let id = chunks.voxel_global(xyz).map_or(0, |v| v.id as usize);
            let emission = BLOCKS()[id].emission();
            if emission.iter().any(|e| *e > 0) {
                println!("{:?}", emission);
                self.add_with_emission_rgb(chunks, xyz.0, xyz.1, xyz.2, emission);
            }
        }
        self.solve_rgb(chunks);

        for (ly, lz, lx) in iproduct!(-1..=CHUNK_SIZE as i32, -1..=CHUNK_SIZE as i32, -1..=CHUNK_SIZE as i32) {
            if lx != -1 && lx != CHUNK_SIZE as i32
              && lz != -1 && lz != CHUNK_SIZE as i32
              && ly != -1 && ly != CHUNK_SIZE as i32 {
                continue;
            }
            let x = cx*CHUNK_SIZE as i32 + lx;
            let y = cy*CHUNK_SIZE as i32 + ly;
            let z = cz*CHUNK_SIZE as i32 + lz;
            if chunks.get_light((x, y, z).into()).0 > 0 {
                self.add_rgbs(chunks, x, y, z);
            }
            self.solve_rgbs(chunks);
        }
    }


    pub fn on_block_break(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32) {
        self.remove_rgb(chunks, x, y, z);
        self.solve_rgb(chunks);

        if chunks.get_sun((x, y+1, z).into()) == MAX_LIGHT || (y+1) as usize == WORLD_HEIGHT*CHUNK_SIZE {
            for i in (0..=y).rev() {
                if chunks.voxel_global((x, i, z).into()).map_or(true, |v| v.id != 0) {break};
                self.solver_sun.add_with_emission(chunks, x, i, z, MAX_LIGHT as u8);
            }
        }

        for (ax, ay, az) in SIDE_COORDS_OFFSET {
            self.add_rgbs(chunks, x+ax, y+ay, z+az);
        }

        self.solve_rgbs(chunks);
    }


    pub fn on_block_set(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32, id: u32) {
        let emission = &BLOCKS()[id as usize].emission();
        self.remove_rgbs(chunks, x, y, z);
        self.solver_sun.solve(chunks);

        for ny in (0..y).rev() {
            if chunks.voxel_global((x, ny, z).into()).map_or(0, |v| v.id) != 0 {break};
            self.solver_sun.remove(chunks, x, ny, z);
            self.solver_sun.solve(chunks);
        }

        if emission.iter().any(|e| *e > 0) {
            self.add_with_emission_rgb(chunks, x, y, z, emission);
        }
        self.solve_rgb(chunks);
    }


    pub fn add_rgb(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32) {
        self.solver_red.add(chunks, x, y, z);
        self.solver_green.add(chunks, x, y, z);
        self.solver_blue.add(chunks, x, y, z);
    }

    pub fn add_rgbs(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32) {
        self.add_rgb(chunks, x, y, z);
        self.solver_sun.add(chunks, x, y, z);
    }

    pub fn add_with_emission_rgb(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32, emission: &[u8; 3]) {
        self.solver_red.add_with_emission(chunks, x, y, z, emission[0]);
        self.solver_green.add_with_emission(chunks, x, y, z, emission[1]);
        self.solver_blue.add_with_emission(chunks, x, y, z, emission[2]);
    }

    pub fn solve_rgb(&mut self, chunks: &mut Chunks) {
        self.solver_red.solve(chunks);
        self.solver_green.solve(chunks);
        self.solver_blue.solve(chunks);
    }

    pub fn solve_rgbs(&mut self, chunks: &mut Chunks) {
        self.solve_rgb(chunks);
        self.solver_sun.solve(chunks);
    }

    pub fn remove_rgb(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32) {
        self.solver_red.remove(chunks, x, y, z);
        self.solver_green.remove(chunks, x, y, z);
        self.solver_blue.remove(chunks, x, y, z);
    }

    pub fn remove_rgbs(&mut self, chunks: &mut Chunks, x: i32, y: i32, z: i32) {
        self.remove_rgb(chunks, x, y, z);
        self.solver_sun.remove(chunks, x, y, z);
    }
}