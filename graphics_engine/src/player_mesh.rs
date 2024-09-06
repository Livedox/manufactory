use std::{collections::HashMap, ops::Add};

use bytemuck::NoUninit;
use itertools::Itertools;
use wgpu::util::DeviceExt;

use crate::vertices::player::PlayerVertex;

use super::{state::State, vertices::{animated_model_instance::AnimatedModelInstance, block_vertex::BlockVertex, model_instance::ModelInstance}};
const INDEX: wgpu::BufferUsages = wgpu::BufferUsages::INDEX;
const VERTEX: wgpu::BufferUsages = wgpu::BufferUsages::VERTEX;
const STORAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE;
const COPY_DST: wgpu::BufferUsages = wgpu::BufferUsages::COPY_DST;

pub fn new_buffer<A: NoUninit>(
    device: &wgpu::Device,
    a: &[A],
    usage: wgpu::BufferUsages,
    label: &str
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(a),
        usage
    })
}

#[derive(Debug)]
pub struct MeshBuffer {
    pub id: u32,
    pub size: usize,
    pub buffer: wgpu::Buffer,
}

impl MeshBuffer {
    pub fn new(id: u32, size: usize, buffer: wgpu::Buffer) -> Self {
        Self {id, size, buffer}
    }
}

#[derive(Debug)]
pub struct PlayerMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

impl PlayerMesh {
    pub fn new(state: &State, player_position: [f32; 3]) -> Self {
        let device = state.device();

        let pp = player_position;
        let c = [
            [pp[0]-0.25, pp[1]-1.75, pp[2]-0.25], [pp[0]+0.25, pp[1]-1.75, pp[2]-0.25],
            [pp[0]-0.25, pp[1]-1.75, pp[2]+0.25], [pp[0]+0.25, pp[1]-1.75, pp[2]+0.25],
            [pp[0]-0.25, pp[1], pp[2]-0.25], [pp[0]+0.25, pp[1], pp[2]-0.25],
            [pp[0]-0.25, pp[1], pp[2]+0.25], [pp[0]+0.25, pp[1], pp[2]+0.25]];
        let light = [1.0, 1.0, 1.0, 1.0];
        let uv = [[0.0, 0.5], [0.5, 0.5], [0.0, 1.0], [0.5, 1.0],
                  [0.0, 0.0], [0.5, 0.0], [0.0, 0.5], [0.5, 0.5],
                  [0.5, 0.0], [0.5, 1.0], [1.0, 0.0], [1.0, 1.0]];
        let k = |c, uv| PlayerVertex::new(c, uv, light);
        let vertices = vec![
            k(c[0], uv[9]), k(c[1], uv[11]), k(c[4], uv[8]),     k(c[5], uv[10]), k(c[4], uv[8]), k(c[1], uv[11]),
            k(c[0], uv[11]), k(c[6], uv[8]), k(c[2], uv[9]),     k(c[0], uv[11]), k(c[4], uv[10]), k(c[6], uv[8]),
            k(c[6], uv[10]), k(c[7], uv[8]), k(c[3], uv[9]),     k(c[3], uv[9]), k(c[2], uv[11]), k(c[6], uv[10]),
            k(c[3], uv[11]), k(c[7], uv[10]), k(c[5], uv[8]),     k(c[5], uv[8]), k(c[1], uv[9]), k(c[3], uv[11]),
            k(c[4], uv[4]), k(c[7], uv[7]), k(c[6], uv[5]),     k(c[4], uv[4]), k(c[5], uv[6]), k(c[7], uv[7]),
            k(c[0], uv[0]), k(c[2], uv[2]), k(c[3], uv[3]),     k(c[3], uv[3]), k(c[1], uv[1]), k(c[0], uv[0])];


        let vertex_buffer = new_buffer(device, &vertices, VERTEX,
            &format!("Player vertex Buffer"));
        
        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
        }
    }
}