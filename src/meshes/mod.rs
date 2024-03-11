use std::{collections::HashMap, sync::{Mutex, Arc}};

use itertools::Itertools;

use crate::{content::Content, engine::{mesh::Mesh, vertices::{animated_model_instance::AnimatedModelInstance, model_instance::ModelInstance}}, graphic::render::{animated_model::AnimatedModelRenderResult, model::ModelRenderResult, RenderResult}, models::animated_model::AnimatedModel, state::State, voxels::block::block_type::BlockType, world::World};


pub struct MeshesRenderInput<'a> {
    pub state: &'a State<'a>,
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
        let MeshesRenderInput {state, render_result} = input;
        let mesh = Mesh::new(state, render_result.mesh, index);
        if index+1 > self.meshes.len() { self.meshes.resize_with(index+1, || {None}) };
        self.meshes[index] = Some(Arc::new(mesh));
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
            let Some(Some(chunk)) = unsafe {&*world.chunks.chunks.get()}.get(*index).cloned() else { return };
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