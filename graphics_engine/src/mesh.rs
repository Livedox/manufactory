use std::collections::HashMap;

use bytemuck::NoUninit;
use itertools::Itertools;
use wgpu::util::DeviceExt;

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

#[derive(Debug, Clone)]
pub struct AnimatedModelRenderResult {
    pub position: [f32; 3],
    pub light: [f32; 4],
    pub progress: f32,
    pub rotation_index: u32,
}

pub type Models = HashMap<u32, Vec<ModelInstance>>;
pub type AnimatedModels = HashMap<u32, Vec<AnimatedModelRenderResult>>;

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

    pub belt_vertex_buffer: wgpu::Buffer,
    pub belt_index_buffer: wgpu::Buffer,
    pub belt_vertex_count: u32,
    pub belt_index_count: u32,

    pub glass_vertex_buffer: wgpu::Buffer,
    pub glass_index_buffer: wgpu::Buffer,
    pub glass_vertex_count: u32,
    pub glass_index_count: u32,

    pub models: Vec<MeshBuffer>,

    pub animated_models: Vec<MeshBuffer>,
    pub transformation_matrices_buffer: Option<wgpu::Buffer>,
    pub transformation_matrices_bind_group: Option<wgpu::BindGroup>
}

impl Mesh {
    pub fn new(state: &State, input: MeshInput, index: usize) -> Self {
        let device = state.device();
        println!("block_vertices: {}", input.block_vertices.len());
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

        let mut animation: Vec<u8> = vec![];
        let mut start_matrix: u32 = 0;
        let animated_models: Vec<MeshBuffer> = input.animated_models.into_iter().sorted_by_key(|(id, _)| *id).map(|(id, data)| {
            let animated_model = state.resources().animated_models().get(id as usize).unwrap();
            let animated_model_instances: Vec::<AnimatedModelInstance> = 
                data.into_iter().map(|AnimatedModelRenderResult {position, light, progress, rotation_index}| {
                    animation.extend(animated_model.calculate_bytes_transforms(None, progress));
                    let instance = AnimatedModelInstance {
                        position,
                        start_matrix,
                        light,
                        rotation_matrix_index: rotation_index,
                    };
                    start_matrix += animated_model.joint_count() as u32;
                    instance
                }).collect();
            let buffer = new_buffer(device, &animated_model_instances, VERTEX, 
                &format!("Instance buffer, animated model id: {}, chunk id: {}", id, index));
            MeshBuffer::new(id, animated_model_instances.len(), buffer)
        }).collect();

        let mut animated_model_buffer = None;
        let mut animated_model_bind_group = None;
        if !animation.is_empty() {
            let buffer = new_buffer(device, &animation, STORAGE | COPY_DST,
                &format!("Transformation matrices storage buffer (Chunk: {})", index));
            
            animated_model_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &state.layouts.transforms_storage,
                entries: &[wgpu::BindGroupEntry {binding: 0, resource: buffer.as_entire_binding()}],
                label: Some(&format!("Transformation matrices bind group (Chunk: {})", index)),
            }));
            animated_model_buffer = Some(buffer);
        }
        
        Self {
            animated_models,
            block_index_buffer,
            block_index_count: input.block_indices.len() as u32,
            block_vertex_buffer,
            block_vertex_count: input.block_vertices.len() as u32,
            glass_index_buffer,
            glass_index_count: input.glass_indices.len() as u32,
            glass_vertex_buffer,
            glass_vertex_count: input.glass_vertices.len() as u32,
            models,
            transformation_matrices_buffer: animated_model_buffer,
            transformation_matrices_bind_group: animated_model_bind_group,
            belt_index_buffer,
            belt_index_count: input.belt_indices.len() as u32,
            belt_vertex_buffer,
            belt_vertex_count: input.belt_vertices.len() as u32,
        }
    }


    pub fn update_transforms_buffer(&self, state: &State, progress: &[(u32, Vec<f32>)]) {
        let Some(buffer) = &self.transformation_matrices_buffer else {return};

        let transforms = progress.iter().flat_map(|(id, p)| {
            let model = state.resources().animated_models().get(*id as usize).unwrap();
            p.iter().flat_map(|progress| {model.calculate_bytes_transforms(None, *progress)})
        }).collect_vec();

        if buffer.size() < transforms.len() as u64 {return};
        state.queue().write_buffer(buffer, 0, transforms.as_slice());
    }
}