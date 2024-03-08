pub(crate) fn get(
    device: &wgpu::Device,
    post_process_bgl: &wgpu::BindGroupLayout,
    color: &wgpu::TextureView,
    reveal: &wgpu::TextureView,
  ) -> wgpu::BindGroup {
      device.create_bind_group(&wgpu::BindGroupDescriptor {
          layout: post_process_bgl,
          entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(color),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(reveal),
            },
          ],
          label: Some("post_process_bg"),
      })
  }