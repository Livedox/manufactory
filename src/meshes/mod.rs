use std::{collections::HashMap, fmt::format, borrow::BorrowMut};

use itertools::Itertools;
use wgpu::{util::DeviceExt, Buffer};
use nalgebra_glm as glm;

use crate::{graphic::render::VoxelRenderer, voxels::chunks::Chunks, vertices::{model_instance::ModelInstance, animated_model_vertex::AnimatedModelVertex, animated_model_instance::AnimatedModelInstance}, model::animated_model::AnimatedModel};

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


#[derive(Debug)]
pub struct Meshes {
    meshes: Vec<Option<Mesh>>,
}

impl Meshes {
    pub fn new() -> Self { Self {meshes: vec![]} }

    pub fn render(&mut self, device: &wgpu::Device, animated_model_layout: &wgpu::BindGroupLayout, renderer: &mut VoxelRenderer, chunks: &mut Chunks, index: usize, all_animated_models: &HashMap<String, AnimatedModel>) {
        let mesh = renderer.render_test(index, chunks);
        let mut models = HashMap::<String, (wgpu::Buffer, usize)>::new();
        let mut animated_models = HashMap::<String, (wgpu::Buffer, usize)>::new();


        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Block vertex Buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&mesh.0),
            usage: wgpu::BufferUsages::VERTEX,
        });


        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Block index Buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&mesh.1),
            usage: wgpu::BufferUsages::INDEX,
        });

        let transport_belt_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Transport belt vertex Buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&mesh.4),
            usage: wgpu::BufferUsages::VERTEX,
        });


        let transport_belt_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Transport belt index buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&mesh.5),
            usage: wgpu::BufferUsages::INDEX,
        });


        let mut animation: Vec<u8> = vec![];
        let mut start_matrix: u32 = 0;
        mesh.2.iter().sorted_by_key(|(name, _)| name.clone()).for_each(|(name, data)| {
            let mut animated_model_instances = Vec::<AnimatedModelInstance>::new();
            let animated_model = all_animated_models.get(name).unwrap();
            data.iter().for_each(|(position, light, progress, rotation_index)| {
                animated_model_instances.push(AnimatedModelInstance {
                    position: position.clone(),
                    start_matrix,
                    light: light.clone(),
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
                contents: bytemuck::cast_slice(&animated_model_instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }), data.len()));
        });

        let mut animated_model_buffer = None;
        let mut animated_model_bind_group = None;
        if animation.len() > 0 {
            animated_model_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Transformation matrices storage buffer (Chunk: {})", index)),
                contents: &animation.as_slice(),
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

        
        mesh.3.iter().for_each(|(name, positions)| {
            let mut model_instances = Vec::<ModelInstance>::new();
            positions.iter().for_each(|(position, light, rotation_index)| {
                model_instances.push(ModelInstance { position: position.clone(), light: light.clone(), rotation_index: *rotation_index })
            });
            models.insert(name.to_string(), (device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Model ({}) instance buffer (Chunk: {})", name, index)),
                contents: bytemuck::cast_slice(&model_instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }), positions.len()));
        }); 


        if index+1 > self.meshes.len() { self.meshes.resize_with(index+1, || {None}) };
        self.meshes[index] = Some(Mesh {
            block_vertex_buffer: vertex_buffer,
            block_index_buffer: index_buffer,
            block_vertex_count: mesh.0.len() as u32,
            block_index_count: mesh.1.len() as u32,

            models,

            animated_models,
            transformation_matrices_buffer: animated_model_buffer,
            transformation_matrices_bind_group: animated_model_bind_group,

            transport_belt_vertex_buffer,
            transport_belt_index_buffer,
            transport_belt_vertex_count: mesh.4.len() as u32,
            transport_belt_index_count: mesh.5.len() as u32,
        });
    }



    pub fn meshes(&self) -> &Vec<Option<Mesh>> {
        &self.meshes
    }

    pub fn mut_meshes(&mut self) -> &mut Vec<Option<Mesh>> {
        &mut self.meshes
    }

    pub fn render_all(
        &mut self,
        chunks: &mut Chunks,
        device: &wgpu::Device,
        animated_model_layout: &wgpu::BindGroupLayout,
        all_animated_models: &HashMap<String, AnimatedModel>,
    ) {
        // loop {
        //     let index = chunks.get_nearest_chunk_index();
        //     if let Some(index) = index {
        //         self.render(device, animated_model_layout, chunks, index, animated_models);
        //     } else {
        //         break;
        //     }
        // }
    }
}