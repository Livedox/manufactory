use wgpu::util::DeviceExt;

use crate::{graphic::render::VoxelRenderer, voxels::chunks::Chunks};


pub type VertexWgpuBuffer = wgpu::Buffer;
pub type IndexWgpuBuffer = wgpu::Buffer;
pub type VertexLen = usize;
pub type IndexLen = usize;
#[derive(Debug)]
pub struct Mesh(pub VertexWgpuBuffer, pub IndexWgpuBuffer, pub VertexLen, pub IndexLen);
#[derive(Debug)]
pub struct Meshes {
    block: Vec<Option<Mesh>>,
}


impl Meshes {
    pub fn new() -> Self { Self {block: vec![]} }

    pub fn render(&mut self, device: &wgpu::Device, renderer: &mut VoxelRenderer, chunks: &mut Chunks, index: usize) {
        let mesh = renderer.render_test(index, chunks);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.0),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Block index Buffer"),
            contents: bytemuck::cast_slice(&mesh.1),
            usage: wgpu::BufferUsages::INDEX,
        });

        if index+1 > self.block.len() { self.block.resize_with(index+1, || {None}) };
        self.block[index] = Some(Mesh(vertex_buffer, index_buffer, mesh.0.len(), mesh.1.len()))
    }

    pub fn block(&self) -> &Vec<Option<Mesh>> {
        &self.block
    }
}