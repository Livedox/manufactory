use std::{collections::HashMap, sync::{Mutex, Arc}};

use itertools::Itertools;
use wgpu::util::DeviceExt;

use crate::{content::Content, engine::vertices::{model_instance::ModelInstance, animated_model_instance::AnimatedModelInstance}, graphic::render::{RenderResult, animated_model::AnimatedModelRenderResult, model::ModelRenderResult}, models::animated_model::AnimatedModel, state::State, voxels::block::{block_type::BlockType}, world::World};

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

    pub models: HashMap<u32, (wgpu::Buffer, usize)>,

    pub animated_models: HashMap<u32, (wgpu::Buffer, usize)>,
    pub transformation_matrices_buffer: Option<wgpu::Buffer>,
    pub transformation_matrices_bind_group: Option<wgpu::BindGroup>
}


pub struct MeshesRenderInput<'a> {
    pub device: &'a wgpu::Device,
    pub animated_model_layout: &'a wgpu::BindGroupLayout,
    pub all_animated_models: &'a Box<[AnimatedModel]>,
    pub render_result: RenderResult,
}


#[derive(Debug)]
pub struct Meshes {
    content: Arc<Content>,
    meshes: Vec<Option<Arc<Mesh>>>,
    // Indicates how many translate need to be performed.
    // Use atomicity if I add this to another thread
    pub need_translate: Arc<Mutex<usize>>, 
}

impl Meshes {
    pub fn new(content: Arc<Content>) -> Self { Self {
        content,
        meshes: vec![],
        need_translate: Arc::new(Mutex::new(0)),
    }}

    pub fn render(&mut self, input: MeshesRenderInput, index: usize) {
        let MeshesRenderInput {device, animated_model_layout, all_animated_models, render_result} = input;

        let mut models = HashMap::<u32, (wgpu::Buffer, usize)>::new();
        let mut animated_models = HashMap::<u32, (wgpu::Buffer, usize)>::new();


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


        let glass_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Glass vertex Buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&render_result.glass_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let glass_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Glass index buffer (Chunk: {})", index)),
            contents: bytemuck::cast_slice(&render_result.glass_indices),
            usage: wgpu::BufferUsages::INDEX,
        });


        let mut animation: Vec<u8> = vec![];
        let mut start_matrix: u32 = 0;
        render_result.animated_models.iter().sorted_by_key(|(id, _)| *id).for_each(|(id, data)| {
            let mut animated_model_instances = Vec::<AnimatedModelInstance>::new();
            let animated_model = all_animated_models.get(*id as usize).unwrap();
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
            animated_models.insert(*id, (device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Model ({}) instance buffer (Chunk: {})", id, index)),
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

        
        render_result.models.iter().for_each(|(id, positions)| {
            let mut model_instances = Vec::<ModelInstance>::new();
            positions.iter().for_each(|ModelRenderResult {position, light, rotation_index}| {
                model_instances.push(ModelInstance { position: *position, light: *light, rotation_index: *rotation_index })
            });
            models.insert(*id, (device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Model ({}) instance buffer (Chunk: {})", id, index)),
                contents: bytemuck::cast_slice(model_instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }), positions.len()));
        }); 


        if index+1 > self.meshes.len() { self.meshes.resize_with(index+1, || {None}) };
        self.meshes[index] = Some(Arc::new(Mesh {
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

            glass_vertex_buffer,
            glass_index_buffer,
            glass_vertex_count: render_result.glass_vertices.len() as u32,
            glass_index_count: render_result.glass_indices.len() as u32,
        }));
    }


    pub fn translate(&mut self, indices: &[(usize, usize)]) {
        let max = *indices.iter().map(|(a, b)| a.max(b)).max().unwrap_or(&0);
        if self.meshes.len() <= max {self.meshes.resize_with(max+1, || None)}
        let mut new_meshes = Vec::<Option<Arc<Mesh>>>::with_capacity(self.meshes.len());
        new_meshes.resize_with(self.meshes.len(), || None);

        for (old, new) in indices.iter() {
            new_meshes[*new] = self.meshes[*old].take();
        }

        self.meshes = new_meshes;
    }


    pub fn update_transforms_buffer(&mut self, state: &State, world: &World, indices: &[usize]) {
        indices.iter().for_each(|index| {
            let Some(Some(chunk)) = unsafe {&*world.chunks.chunks.get()}.get(*index).map(|c| c.clone()) else { return };
            if chunk.live_voxels.0.read().unwrap().is_empty() {return};
            let mut transforms_buffer: Vec<u8> = vec![];
            let mut animated_models: HashMap<u32, Vec<f32>> = HashMap::new();
    
            chunk.live_voxels.0.read().unwrap().iter().sorted_by_key(|data| {data.0}).for_each(|data| {
                let progress = data.1.live_voxel.animation_progress();
                let block_type = &self.content.blocks[data.1.id as usize].block_type();
                if let BlockType::AnimatedModel {id} = block_type {
                    if let Some(animated_model) = animated_models.get_mut(id) {
                        animated_model.push(progress);
                    } else {
                        animated_models.insert(*id, vec![progress]);
                    }
                } else if let BlockType::ComplexObject { id } = block_type {
                    world.chunks.content.complex_objects[*id as usize].animated_models.iter().for_each(|id| {
                        if let Some(animated_model) = animated_models.get_mut(id) {
                            animated_model.push(progress);
                        } else {
                            animated_models.insert(*id, vec![progress]);
                        }
                    });
                }
            });
    
            animated_models.iter().sorted_by_key(|(id, _)| *id).for_each(|(id, progress_vec)| {
                let model = state.animated_models.get(*id as usize).unwrap();
                progress_vec.iter().for_each(|progress| {
                    transforms_buffer.extend(model.calculate_bytes_transforms(None, *progress));
                });
            });
    
            if let Some(Some(mesh)) = &mut self.meshes().get(*index) {
                let Some(buffer) = &mesh.transformation_matrices_buffer else {return};
                if buffer.size() >= transforms_buffer.len() as u64 {
                    state.queue().write_buffer(buffer, 0, transforms_buffer.as_slice());
                }
            }
        });
    }

    pub fn meshes(&self) -> &[Option<Arc<Mesh>>] {
        &self.meshes
    }

    pub fn is_need_translate(&self) -> bool {
        *self.need_translate.lock().unwrap() != 0
    }

    pub fn sub_need_translate(&mut self) {
        *self.need_translate.lock().unwrap() -= 1;
    }
}