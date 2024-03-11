use std::collections::HashMap;

use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::graphic::render::{animated_model::AnimatedModels, model::{ModelRenderResult, Models}};
use super::{state::{self, State}, vertices::{block_vertex::BlockVertex, model_instance::ModelInstance}};
const INDEX: wgpu::BufferUsages = wgpu::BufferUsages::INDEX;
const VERTEX: wgpu::BufferUsages = wgpu::BufferUsages::VERTEX;

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

pub struct MeshInput {
    pub block_vertices: Vec<BlockVertex>,
    pub block_indices: Vec<u32>,
    pub glass_vertices: Vec<BlockVertex>,
    pub glass_indices: Vec<u32>,
    pub belt_vertices: Vec<BlockVertex>,
    pub belt_indices: Vec<u32>,

    pub models: Models,
    pub animated_models: AnimatedModels,
}

#[derive(Debug)]
pub struct Mesh {
    pub block_vertex_buffer: wgpu::Buffer,
    pub block_index_buffer: wgpu::Buffer,
    pub block_vertex_count: u32,
    pub block_index_count: u32,

    pub transport_belt_vertex_buffer: wgpu::Buffer,
    pub transport_belt_index_buffer: wgpu::Buffer,
    pub transport_belt_vertex_count: u32,
    pub transport_belt_index_count: u32,

    pub glass_vertex_buffer: wgpu::Buffer,
    pub glass_index_buffer: wgpu::Buffer,
    pub glass_vertex_count: u32,
    pub glass_index_count: u32,

    pub models: Vec<MeshBuffer>,

    pub animated_models: HashMap<u32, (wgpu::Buffer, usize)>,
    pub transformation_matrices_buffer: Option<wgpu::Buffer>,
    pub transformation_matrices_bind_group: Option<wgpu::BindGroup>
}

impl Mesh {
    pub fn new(state: &State, input: MeshInput, index: usize) {
        let device = state.device();

        let block_vertex_buffer = new_buffer(device, &input.block_vertices, VERTEX,
            &format!("Block vertex Buffer (Chunk: {})", index));
        let block_index_buffer = new_buffer(device, &input.block_indices, INDEX,
            &format!("Block index Buffer (Chunk: {})", index));

        let belt_vertex_buffer = new_buffer(device, &input.belt_vertices, VERTEX,
            &format!("Transport belt vertex Buffer (Chunk: {})", index));
        let belt_index_buffer = new_buffer(device, &input.belt_indices, INDEX,
            &format!("Transport belt index buffer (Chunk: {})", index));

        let glass_vertex_buffer = new_buffer(device, &input.glass_vertices, VERTEX,
            &format!("Glass vertex Buffer (Chunk: {})", index));
        let glass_index_buffer = new_buffer(device, &input.glass_indices, INDEX,
            &format!("Glass index buffer (Chunk: {})", index));
        
        let models: Vec<MeshBuffer> = input.models.into_iter().map(|(id, render_results)| {
            let buffer = new_buffer(device, &render_results, VERTEX, 
                &format!("Instance buffer, model id: {}, chunk id: {}", id, index));
            MeshBuffer::new(id, render_results.len(), buffer)
        }).collect();
    }
}