pub(crate) struct Shaders {
    pub(crate) block: wgpu::ShaderModule,
    pub(crate) transport_belt: wgpu::ShaderModule,
    pub(crate) model: wgpu::ShaderModule,
    pub(crate) animated_model: wgpu::ShaderModule,
    pub(crate) selection: wgpu::ShaderModule,
    pub(crate) crosshair: wgpu::ShaderModule,
    pub(crate) post_proccess_test: wgpu::ShaderModule,
}

impl Shaders {
    pub(crate) fn new(device: &wgpu::Device) -> Self {Self{
        block: device.create_shader_module(wgpu::include_wgsl!("block.wgsl")),
        transport_belt: device.create_shader_module(wgpu::include_wgsl!("transport_belt.wgsl")),
        model: device.create_shader_module(wgpu::include_wgsl!("model.wgsl")),
        animated_model: device.create_shader_module(wgpu::include_wgsl!("animated_model.wgsl")),
        selection: device.create_shader_module(wgpu::include_wgsl!("selection.wgsl")),
        crosshair: device.create_shader_module(wgpu::include_wgsl!("crosshair.wgsl")),
        post_proccess_test: device.create_shader_module(wgpu::include_wgsl!("post_proccess_test.wgsl")),
    }}
}