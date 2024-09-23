use wgpu::PipelineCompilationOptions;

use crate::constants::DEPTH_FORMAT;

pub enum PipelineBuilderShader<'a> {
    General{shader: &'a wgpu::ShaderModule},
    Separated {vertex: &'a wgpu::ShaderModule, fragment: &'a wgpu::ShaderModule}
}

impl<'a> PipelineBuilderShader<'a> {
    pub fn new(shader: &'a wgpu::ShaderModule) -> Self {
        Self::General { shader }
    }

    pub fn new_separated(vertex: &'a wgpu::ShaderModule, fragment: &'a wgpu::ShaderModule) -> Self {
        Self::Separated { vertex, fragment }
    }
}

pub struct PipelineBuilder<'a> {
    device: &'a wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    buffers: &'a [wgpu::VertexBufferLayout<'a>],

    targets: Option<&'a [Option<wgpu::ColorTargetState>]>,
    vertex: &'a wgpu::ShaderModule,
    fragment: &'a wgpu::ShaderModule, 
    depth_write_enabled: bool,
    is_depth: bool,
    blend: wgpu::BlendState,
    sample_count: u32,
    topology: wgpu::PrimitiveTopology,
    label: &'a str
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(
        device: &'a wgpu::Device,
        format: wgpu::TextureFormat,
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
        buffers: &'a [wgpu::VertexBufferLayout<'a>],
        shaders: PipelineBuilderShader<'a>,
    ) -> Self {
        let (vertex, fragment) = match shaders {
            PipelineBuilderShader::General { shader } => (shader, shader),
            PipelineBuilderShader::Separated { vertex, fragment } => (vertex, fragment)
        };

        Self {
            device,
            bind_group_layouts,
            buffers,
            format,
            is_depth: true,
            depth_write_enabled: true,
            vertex,
            fragment,
            sample_count: 1,
            label: "unknown",
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: wgpu::BlendState::REPLACE,
            targets: None
        }
    }

    pub fn sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    pub fn topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn blend(mut self, blend: wgpu::BlendState) -> Self {
        self.blend = blend;
        self
    }

    pub fn is_depth(mut self, is_depth: bool) -> Self {
        self.is_depth = is_depth;
        self
    }

    pub fn depth_write_enabled(mut self, depth_write_enabled: bool) -> Self {
        self.depth_write_enabled = depth_write_enabled;
        self
    }

    /// Carefully replaces the default builder value of wgpu::FragmentState {targets, ..}
    pub fn fragment_targets(mut self, targets: &'a [Option<wgpu::ColorTargetState>]) -> Self {
        self.targets = Some(targets);
        self
    }

    pub fn build(self) -> wgpu::RenderPipeline {
        let render_pipeline_layout = 
            self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("Render Pipeline Layout ({})", self.label)),
                bind_group_layouts: self.bind_group_layouts,
                push_constant_ranges: &[],
            });
        
        let default_targets = &[Some(wgpu::ColorTargetState {
            format: self.format,
            blend: Some(self.blend),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let fragment_targets = self.targets.unwrap_or(default_targets);

        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline ({})", self.label)),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: self.vertex,
                entry_point: "vs_main",
                buffers: self.buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: self.fragment,
                entry_point: "fs_main",
                targets: fragment_targets,
                compilation_options: Default::default(),
            }),
    
            primitive: wgpu::PrimitiveState {
                topology: self.topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.is_depth.then(|| wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: self.depth_write_enabled,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: self.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}

