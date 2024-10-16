use std::simd::Simd;

use crate::{content::Content, light::light::Light, voxels::{block::block_test::BlockBase, chunk::Chunk, chunks::{Chunks, WORLD_BLOCK_HEIGHT}}};
use super::block_managers::BlockManagers;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockFaceLight([Light; 9]);

impl BlockFaceLight {
    #[inline]
    pub fn new(chunks: &Chunks, coords: [(i32, i32, i32); 9]) -> Self {
        Self(coords.map(|coord| chunks.get_light(coord.into())))
    }

    const ANGLE_INDICES: [[usize; 3]; 4] = [
        [1,   0,   3],
        [1,   2,   5],
        [7,   8,   5],
        [7,   6,   3]
    ];
    const CENTER_INFLUENCE: f32 = 1.5;
    const COEFFICIENT: f32 = Self::CENTER_INFLUENCE + 3.0;
    pub fn get(&self) -> [[f32; 4]; 4] {
        let normalized: [Simd<f32, 4>; 9] = self.0.clone().map(|l| l.get_normalized().into());
        let center = &normalized[4] * Simd::<f32, 4>::from_array([Self::CENTER_INFLUENCE; 4]);

        Self::ANGLE_INDICES.map(|idx| {
            (unsafe {
                (normalized.get_unchecked(idx[0]) +
                normalized.get_unchecked(idx[1]) +
                normalized.get_unchecked(idx[2]) +
                center) /
                Simd::<f32, 4>::from_array([Self::COEFFICIENT; 4])
            }).into()
        })
    }
}

#[derive(Debug, Clone)]
pub struct BlockFace {
    pub layer: u32,
    pub light: BlockFaceLight,
    pub size: [u8; 2],
}

impl BlockFace {
    #[inline]
    pub fn new(layer: u32, light: BlockFaceLight) -> Self {
        Self { layer, light, size: [1, 1] }
    }
}

#[inline]
pub fn render_block(
    content: &Content,
    block_manager: &mut BlockManagers,
    chunks: &Chunks,
    chunk: &Chunk,
    block: &BlockBase,
    faces: &[u32; 6],
    local: (usize, usize, usize),
) {
    let (lx, ly, lz) = local;
    let (x, y, z) = chunk.coord.to_global((lx, ly, lz).into()).into();
    let (nx, px, ny, py, nz, pz) = (x-1, x+1, y-1, y+1, z-1, z+1);
    if !is_blocked(x-1, y, z, chunks, block, content) {
        let light = BlockFaceLight::new(chunks, [
            (nx, ny, nz), (nx, y, nz), (nx, py, nz),
            (nx, ny,  z), (nx, y, z),  (nx, py, z),
            (nx, ny, pz), (nx, y, pz), (nx, py, pz)
        ]);
        block_manager.set(0, lx, ly, lz, BlockFace::new(faces[0], light));
    }

    if !is_blocked(x+1, y, z, chunks, block, content) {
        let light = BlockFaceLight::new(chunks, [
            (px, ny, nz), (px, y, nz), (px, py, nz),
            (px, ny,  z), (px, y, z),  (px, py, z),
            (px, ny, pz), (px, y, pz), (px, py, pz)
        ]);
        block_manager.set(1, lx, ly, lz, BlockFace::new(faces[1], light));
    }

    if !is_blocked(x, y-1, z, chunks, block, content) {
        let light = BlockFaceLight::new(chunks, [
            (nx, ny, nz), (nx, ny, z), (nx, ny, pz),
            (x,  ny, nz), (x,  ny, z), (x,  ny, pz),
            (px, ny, nz), (px, ny, z), (px, ny, pz)
        ]);
        block_manager.set(2, ly, lx, lz, BlockFace::new(faces[2], light));
    }


    if !is_blocked(x, y+1, z, chunks, block, content) {
        let light = BlockFaceLight::new(chunks, [
            (nx, py, nz), (nx, py, z), (nx, py, pz),
            (x,  py, nz), (x,  py, z), (x,  py, pz),
            (px, py, nz), (px, py, z), (px, py, pz)
        ]);
        block_manager.set(3, ly, lx, lz, BlockFace::new(faces[3], light));
    }

    if !is_blocked(x, y, z-1, chunks, block, content) {
        let light = BlockFaceLight::new(chunks, [
            (nx, ny, nz), (x, ny, nz), (px, ny, nz),
            (nx,  y, nz), (x,  y, nz), (px,  y, nz),
            (nx, py, nz), (x, py, nz), (px, py, nz)
        ]);
        block_manager.set(4, lz, lx, ly, BlockFace::new(faces[4], light));
    }

    if !is_blocked(x, y, z+1, chunks, block, content) {
        let light = BlockFaceLight::new(chunks, [
            (nx, ny, pz), (x, ny, pz), (px, ny, pz),
            (nx,  y, pz), (x,  y, pz), (px,  y, pz),
            (nx, py, pz), (x, py, pz), (px, py, pz)
        ]);
        block_manager.set(5, lz, lx, ly, BlockFace::new(faces[5], light));
    }
}

#[inline]
fn is_blocked(x: i32, y: i32, z: i32, chunks: &Chunks, block: &BlockBase, content: &Content) -> bool {
    if y < 0 || y >= WORLD_BLOCK_HEIGHT as i32 {return false};
    let Some(voxel) = chunks.voxel_global((x, y, z).into()) else {return false};
    let nblock = &content.blocks[voxel.id as usize].base;
    if block.is_glass && nblock.is_glass {
        return block.id == nblock.id;
    }
    !nblock.is_light_passing
}