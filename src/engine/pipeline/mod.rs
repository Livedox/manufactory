use wgpu::{Device, BindGroupLayout, VertexBufferLayout, ShaderModule, TextureFormat, PrimitiveTopology, RenderPipeline};

use crate::engine::texture;

use super::{bind_group_layout::Layouts, shaders::Shaders, vertices::{block_vertex::BlockVertex, model_vertex::ModelVertex, model_instance::ModelInstance, animated_model_vertex::AnimatedModelVertex, animated_model_instance::AnimatedModelInstance, selection_vertex::SelectionVertex}};

pub const IS_LINE: bool = false;
pub const PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology = match IS_LINE {
    true => wgpu::PrimitiveTopology::LineList,
    false => wgpu::PrimitiveTopology::TriangleList,
};

pub(crate) struct Pipelines {
    pub block: RenderPipeline,
    pub transport_belt: RenderPipeline,
    pub model: RenderPipeline,
    pub animated_model: RenderPipeline,
    pub selection: RenderPipeline,
    pub crosshair: RenderPipeline,
    pub post_proccess_test: RenderPipeline,
}

impl Pipelines {
    pub fn new(
      device: &Device,
      layouts: &Layouts,
      shaders: &Shaders,
      format: TextureFormat,
      sample_count: u32
    ) -> Self {Self {
        block: new(
            device, &[&layouts.sun, &layouts.block_texture, &layouts.camera],
            &[BlockVertex::desc()], &shaders.block,
            format, PRIMITIVE_TOPOLOGY,
            sample_count, "block"),
        
        transport_belt: new(
            device, &[&layouts.sun, &layouts.block_texture, &layouts.camera, &layouts.time],
            &[BlockVertex::desc()], &shaders.transport_belt,
            format, PRIMITIVE_TOPOLOGY,
            sample_count, "transport belt"),
        
        model: new(
            device, &[&layouts.sun, &layouts.model_texture, &layouts.camera],
            &[ModelVertex::desc(), ModelInstance::desc()], &shaders.model,
            format, PRIMITIVE_TOPOLOGY,
            sample_count, "model"),
        
        animated_model: new(
            device, &[&layouts.sun, &layouts.model_texture, &layouts.camera, &layouts.transforms_storage],
            &[AnimatedModelVertex::desc(), AnimatedModelInstance::desc()], &shaders.animated_model,
            format, PRIMITIVE_TOPOLOGY,
            sample_count, "animated_model"),
        
        selection: new(
            device, &[&layouts.camera],
            &[SelectionVertex::desc()], &shaders.selection,
            format, wgpu::PrimitiveTopology::LineList,
            sample_count, "selection"),
        
        crosshair: new(
            device, &[&layouts.crosshair_aspect_scale],
            &[], &shaders.crosshair,
            format, wgpu::PrimitiveTopology::TriangleList,
            sample_count, "crosshair"),
        
        post_proccess_test: new(
            device, &[&layouts.post_proccess_test],
            &[], &shaders.post_proccess_test,
            format, wgpu::PrimitiveTopology::TriangleList,
            sample_count, "post_proccess_test"),
    }}
}

pub fn new(
    device: &Device,
    bind_group_layouts: &[&BindGroupLayout],
    buffers: &[VertexBufferLayout<'_>],
    shader: &ShaderModule,
    format: TextureFormat,
    topology: PrimitiveTopology,
    sample_count: u32,
    label: &str,
) -> RenderPipeline {
    let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Render Pipeline Layout ({})", label)),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("Render Pipeline ({})", label)),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),

        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    })
}