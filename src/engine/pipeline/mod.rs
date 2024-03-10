use wgpu::{Device, BindGroupLayout, VertexBufferLayout, ShaderModule, TextureFormat, PrimitiveTopology, RenderPipeline};
use crate::engine::texture;
use self::builder::{PipelineBuilder, PipelineBuilderShader};

use super::{bind_group_layout::Layouts, shaders::Shaders, vertices::{block_vertex::BlockVertex, model_vertex::ModelVertex, model_instance::ModelInstance, animated_model_vertex::AnimatedModelVertex, animated_model_instance::AnimatedModelInstance, selection_vertex::SelectionVertex}};

mod builder;

pub const IS_LINE: bool = false;
pub const PRIMITIVE_TOPOLOGY: wgpu::PrimitiveTopology = match IS_LINE {
    true => wgpu::PrimitiveTopology::LineList,
    false => wgpu::PrimitiveTopology::TriangleList,
};

const CROSSHAIR_BLEND_STATE: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::OneMinusDst,
        dst_factor: wgpu::BlendFactor::Dst,
        operation: wgpu::BlendOperation::Subtract,
    },
    alpha: wgpu::BlendComponent::OVER,
};

const ACCUM_BLEND_STATE: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One, 
        dst_factor: wgpu::BlendFactor::One, 
        operation: wgpu::BlendOperation::Add, 
    }, 
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One, 
        dst_factor: wgpu::BlendFactor::One, 
        operation: wgpu::BlendOperation::Add, 
    }, 
};

const REVEAL_BLEND_STATE: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero, 
        dst_factor: wgpu::BlendFactor::OneMinusSrc, 
        operation: wgpu::BlendOperation::Add, 
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero, 
        dst_factor: wgpu::BlendFactor::OneMinusSrc, 
        operation: wgpu::BlendOperation::Add, 
    }
};

pub(crate) struct Pipelines {
    pub block: RenderPipeline,
    pub transport_belt: RenderPipeline,
    pub model: RenderPipeline,
    pub animated_model: RenderPipeline,
    pub selection: RenderPipeline,
    pub crosshair: RenderPipeline,
    pub post_process: RenderPipeline,
    pub multisampled_post_process: RenderPipeline,
    pub glass: RenderPipeline,
    pub composite: RenderPipeline,
}

impl Pipelines {
    pub fn new(
        device: &Device,
        layouts: &Layouts,
        shaders: &Shaders,
        format: TextureFormat,
        sample_count: u32
    ) -> Self {Self {
        block: PipelineBuilder::new(device, format,
                &[&layouts.sun, &layouts.block_texture, &layouts.camera], &[BlockVertex::desc()],
                PipelineBuilderShader::new_separated(&shaders.block_vertex, &shaders.block_fragment))
            .sample_count(sample_count).topology(PRIMITIVE_TOPOLOGY)
            .label("block").build(),
        
        transport_belt: PipelineBuilder::new(device, format,
                &[&layouts.sun, &layouts.block_texture, &layouts.camera, &layouts.time],
                &[BlockVertex::desc()], PipelineBuilderShader::new(&shaders.transport_belt))
            .sample_count(sample_count).topology(PRIMITIVE_TOPOLOGY)
            .label("transport belt").build(),
        
        model: PipelineBuilder::new(device, format,
                &[&layouts.sun, &layouts.model_texture, &layouts.camera],
                &[ModelVertex::desc(), ModelInstance::desc()], PipelineBuilderShader::new(&shaders.model))
            .sample_count(sample_count).topology(PRIMITIVE_TOPOLOGY)
            .label("model").build(),
        
        animated_model: PipelineBuilder::new(device, format,
                &[&layouts.sun, &layouts.model_texture, &layouts.camera, &layouts.transforms_storage],
                &[AnimatedModelVertex::desc(), AnimatedModelInstance::desc()],
                PipelineBuilderShader::new(&shaders.animated_model))
            .sample_count(sample_count).topology(PRIMITIVE_TOPOLOGY)
            .label("animated_model").build(),
        
        selection: PipelineBuilder::new(device, format,
                &[&layouts.camera], &[SelectionVertex::desc()],
                PipelineBuilderShader::new(&shaders.selection))
            .sample_count(sample_count).topology(wgpu::PrimitiveTopology::LineList)
            .label("selection").build(),
        
        crosshair: PipelineBuilder::new(device, format,
                &[&layouts.crosshair_aspect_scale], &[],
                PipelineBuilderShader::new(&shaders.crosshair))
            .sample_count(sample_count)
            .label("crosshair")
            .is_depth(false)
            .blend(CROSSHAIR_BLEND_STATE).build(),
        
        post_process: PipelineBuilder::new(device, format,
                &[&layouts.post_process], &[],
                PipelineBuilderShader::new_separated(&shaders.fullscreen_vertex,
                    &shaders.post_process_fragment))
            .label("post_process").is_depth(false).build(),
        
        multisampled_post_process: PipelineBuilder::new(device, format,
                &[&layouts.multisampled_post_process], &[],
                PipelineBuilderShader::new_separated(&shaders.fullscreen_vertex,
                    &shaders.multisampled_post_process_fragment))
            .label("multisampled_post_process").is_depth(false)
            .sample_count(sample_count).build(),
        
        glass: PipelineBuilder::new(device, format,
                &[&layouts.sun, &layouts.block_texture, &layouts.camera], &[BlockVertex::desc()],
                PipelineBuilderShader::new_separated(&shaders.block_vertex, &shaders.glass_fragment))
            .label("glass").depth_write_enabled(false)
            .sample_count(sample_count)
            .fragment_targets(&[
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float, 
                    blend: Some(ACCUM_BLEND_STATE), 
                    write_mask: wgpu::ColorWrites::ALL, 
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm, 
                    blend: Some(REVEAL_BLEND_STATE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ])
            .build(),
        
        composite: PipelineBuilder::new(device, format,
                &[&layouts.oit], &[],
                PipelineBuilderShader::new_separated(&shaders.fullscreen_vertex, &shaders.composite_fragment))
            .label("composite").is_depth(false)
            .blend(wgpu::BlendState::ALPHA_BLENDING).build(),
    }}
}