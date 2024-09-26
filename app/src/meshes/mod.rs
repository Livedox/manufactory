use std::{collections::{BTreeMap, HashMap}, sync::{Arc, Mutex}};

use itertools::Itertools;
use graphics_engine::mesh::Mesh;

use crate::{content::Content, graphic::render::{model::ModelRenderResult, RenderResult}, state::State, voxels::{block::block_type::BlockType, new_chunks::ChunkCoord}, world::World};


pub struct MeshesRenderInput<'a> {
    pub state: &'a State<'a>,
    pub render_result: RenderResult,
}


#[derive(Debug)]
pub struct Meshes {
    content: Arc<Content>,
    meshes: HashMap<ChunkCoord, Arc<Mesh>>,
    // Indicates how many translate need to be performed.
    // Use atomicity if I add this to another thread
    pub need_translate: Arc<Mutex<usize>>, 
}

impl Meshes {
    pub fn new(content: Arc<Content>) -> Self { Self {
        content,
        meshes: HashMap::new(),
        need_translate: Arc::new(Mutex::new(0)),
    }}

    pub fn render(&mut self, input: MeshesRenderInput, index: usize) {
        let MeshesRenderInput {state, render_result} = input;
        let mesh = Mesh::new(state, render_result.mesh, index);
        self.meshes.insert(render_result.coord, Arc::new(mesh));
    }


    // pub fn translate(&mut self, indices: &[(usize, usize)]) {
    //     let max = *indices.iter().map(|(a, b)| a.max(b)).max().unwrap_or(&0);
    //     if self.meshes.len() <= max {self.meshes.resize_with(max+1, || None)}
    //     let mut new_meshes = Vec::<Option<Arc<Mesh>>>::with_capacity(self.meshes.len());
    //     new_meshes.resize_with(self.meshes.len(), || None);

    //     for (old, new) in indices.iter() {
    //         new_meshes[*new] = self.meshes[*old].take();
    //     }

    //     self.meshes = new_meshes;
    // }


    pub fn update_transforms_buffer(&mut self, state: &State, world: &World, indices: &[ChunkCoord]) {
        indices.iter().for_each(|cc| {
            let Some(chunk) = unsafe {&*world.chunks.chunks.get()}.get(&cc).cloned() else { return };
            let Some(mesh) = self.meshes().get(&cc) else {return};
            if chunk.live_voxels.is_empty() {return};

            let mut animated_models: BTreeMap<u32, Vec<f32>> = BTreeMap::new();
    
            chunk.live_voxels.0.read().unwrap().iter().sorted_by_key(|data| {data.0}).for_each(|data| {
                let progress = data.1.live_voxel.animation_progress();
                let block_type = &self.content.blocks[data.1.id as usize].block_type();
                if let BlockType::AnimatedModel {id} = block_type {
                    animated_models.entry(*id)
                        .and_modify(|models| models.push(progress))
                        .or_insert(vec![progress]);
                } else if let BlockType::ComplexObject { id } = block_type {
                    world.chunks.content.complex_objects[*id as usize].animated_models.iter().for_each(|id| {
                        animated_models.entry(*id)
                            .and_modify(|models| models.push(progress))
                            .or_insert(vec![progress]);
                    });
                }
            });
    
            mesh.update_transforms_buffer(state, &Vec::from_iter(animated_models.into_iter()));
        });
    }

    pub fn meshes(&self) -> &HashMap<ChunkCoord, Arc<Mesh>> {
        &self.meshes
    }

    pub fn is_need_translate(&self) -> bool {
        *self.need_translate.lock().unwrap() != 0
    }

    pub fn sub_need_translate(&mut self) {
        *self.need_translate.lock().unwrap() -= 1;
    }
}