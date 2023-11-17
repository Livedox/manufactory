use std::{collections::HashMap, sync::{Mutex, Arc}};

use itertools::Itertools;
use wgpu::util::DeviceExt;


use crate::{graphic::render::{render, AnimatedModelRenderResult, ModelRenderResult, RenderResult}, voxels::chunks::Chunks, vertices::{model_instance::ModelInstance, animated_model_instance::AnimatedModelInstance}, models::animated_model::AnimatedModel, world::World};

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

    pub models: HashMap<String, (wgpu::Buffer, usize)>,

    pub animated_models: HashMap<String, (wgpu::Buffer, usize)>,
    pub transformation_matrices_buffer: Option<wgpu::Buffer>,
    pub transformation_matrices_bind_group: Option<wgpu::BindGroup>
}


pub struct MeshesRenderInput<'a> {
    pub device: &'a wgpu::Device,
    pub animated_model_layout: &'a wgpu::BindGroupLayout,
    pub all_animated_models: &'a HashMap<String, AnimatedModel>,
    pub render_result: RenderResult,
}


#[derive(Debug)]
pub struct Meshes {
    meshes: Vec<Option<Mesh>>,
}

impl Meshes {
    pub fn new() -> Self { Self {meshes: vec![]} }

    pub fn render(&mut self, input: MeshesRenderInput) {
        let MeshesRenderInput {device, animated_model_layout, all_animated_models, render_result} = input;
        let index = render_result.chunk_index;

        let mut models = HashMap::<String, (wgpu::Buffer, usize)>::new();
        let mut animated_models = HashMap::<String, (wgpu::Buffer, usize)>::new();


        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Block vertex Buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&render_result.block_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });


        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Block index Buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&render_result.block_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let transport_belt_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Transport belt vertex Buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&render_result.belt_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });


        let transport_belt_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Transport belt index buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&render_result.belt_indices),
            usage: wgpu::BufferUsages::INDEX,
        });


        let mut animation: Vec<u8> = vec![];
        let mut start_matrix: u32 = 0;
        render_result.animated_models.iter().sorted_by_key(|(name, _)| *name).for_each(|(name, data)| {
            let mut animated_model_instances = Vec::<AnimatedModelInstance>::new();
            let animated_model = all_animated_models.get(name).unwrap();
            data.iter().for_each(|AnimatedModelRenderResult {position, light, progress, rotation_index}| {
                animated_model_instances.push(AnimatedModelInstance {
                    position: *position,
                    start_matrix,
                    light: *light,
                    rotation_matrix_index: *rotation_index,
                });
                let transforms = animated_model.calculate_transforms(None, *progress);
                transforms.iter().for_each(|transform| {
                    animation.extend(bytemuck::cast_slice(transform.as_slice()));
                });
                start_matrix += animated_model.joint_count() as u32;
            });
            animated_models.insert(name.to_string(), (device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Model ({}) instance buffer (Chunk: {})", name, index)),
                contents: bytemuck::cast_slice(animated_model_instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }), data.len()));
        });

        let mut animated_model_buffer = None;
        let mut animated_model_bind_group = None;
        if !animation.is_empty() {
            animated_model_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Transformation matrices storage buffer (Chunk: {})", index)),
                contents: animation.as_slice(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }));
            animated_model_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: animated_model_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: animated_model_buffer.as_ref().unwrap().as_entire_binding(),
                }],
                label: Some(&format!("Transformation matrices bind group (Chunk: {})", index)),
            }));
        }

        
        render_result.models.iter().for_each(|(name, positions)| {
            let mut model_instances = Vec::<ModelInstance>::new();
            positions.iter().for_each(|ModelRenderResult {position, light, rotation_index}| {
                model_instances.push(ModelInstance { position: *position, light: *light, rotation_index: *rotation_index })
            });
            models.insert(name.to_string(), (device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Model ({}) instance buffer (Chunk: {})", name, index)),
                contents: bytemuck::cast_slice(model_instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }), positions.len()));
        }); 


        if index+1 > self.meshes.len() { self.meshes.resize_with(index+1, || {None}) };
        self.meshes[index] = Some(Mesh {
            block_vertex_buffer: vertex_buffer,
            block_index_buffer: index_buffer,
            block_vertex_count: render_result.block_vertices.len() as u32,
            block_index_count: render_result.block_indices.len() as u32,

            models,

            animated_models,
            transformation_matrices_buffer: animated_model_buffer,
            transformation_matrices_bind_group: animated_model_bind_group,

            transport_belt_vertex_buffer,
            transport_belt_index_buffer,
            transport_belt_vertex_count: render_result.belt_vertices.len() as u32,
            transport_belt_index_count: render_result.belt_indices.len() as u32,
        });
    }



    pub fn meshes(&self) -> &Vec<Option<Mesh>> {
        &self.meshes
    }

    pub fn mut_meshes(&mut self) -> &mut Vec<Option<Mesh>> {
        &mut self.meshes
    }
}