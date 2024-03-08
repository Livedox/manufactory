pub(crate) fn get(
    device: &wgpu::Device,
    oit_bgl: &wgpu::BindGroupLayout,
    accum: &wgpu::TextureView,
    reveal: &wgpu::TextureView
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: oit_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(accum),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(reveal),
            },
        ],
        label: Some("post_process_bg"),
    })
}