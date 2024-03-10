pub(crate) struct Shaders {
    pub(crate) block_vertex: wgpu::ShaderModule,
    pub(crate) block_fragment: wgpu::ShaderModule,
    pub(crate) glass_fragment: wgpu::ShaderModule,
    pub(crate) transport_belt: wgpu::ShaderModule,
    pub(crate) model: wgpu::ShaderModule,
    pub(crate) animated_model: wgpu::ShaderModule,
    pub(crate) selection: wgpu::ShaderModule,
    pub(crate) crosshair: wgpu::ShaderModule,

    pub(crate) post_process_fragment: wgpu::ShaderModule,
    pub(crate) multisampled_post_process_fragment: wgpu::ShaderModule,
    pub(crate) composite_fragment: wgpu::ShaderModule,
    pub(crate) fullscreen_vertex: wgpu::ShaderModule,
}

impl Shaders {
    pub(crate) fn new(device: &wgpu::Device) -> Self {Self{
        fullscreen_vertex: device.create_shader_module(wgpu::include_wgsl!("fullscreen_vertex.wgsl")),
        block_vertex: device.create_shader_module(wgpu::include_wgsl!("block_vertex.wgsl")),
        block_fragment: device.create_shader_module(wgpu::include_wgsl!("block_fragment.wgsl")),
        transport_belt: device.create_shader_module(wgpu::include_wgsl!("transport_belt.wgsl")),
        model: device.create_shader_module(wgpu::include_wgsl!("model.wgsl")),
        animated_model: device.create_shader_module(wgpu::include_wgsl!("animated_model.wgsl")),
        selection: device.create_shader_module(wgpu::include_wgsl!("selection.wgsl")),
        crosshair: device.create_shader_module(wgpu::include_wgsl!("crosshair.wgsl")),
        post_process_fragment: device.create_shader_module(wgpu::include_wgsl!("post_process_fragment.wgsl")),
        multisampled_post_process_fragment: device.create_shader_module(
            wgpu::include_wgsl!("multisampled_post_process_fragment.wgsl")),
        glass_fragment: device.create_shader_module(wgpu::include_wgsl!("glass_fragment.wgsl")),
        composite_fragment: device.create_shader_module(wgpu::include_wgsl!("composite_fragment.wgsl")),
    }}
}