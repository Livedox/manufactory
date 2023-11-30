use wgpu::util::DeviceExt;

pub struct BindGroupBuffer {
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}

impl BindGroupBuffer {
    /// Buffer(usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    pub(crate) fn new(device: &wgpu::Device, contents: &[u8], layout: &wgpu::BindGroupLayout, label: &str) -> Self {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&(label.to_owned() + " buffer")),
                contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some(&(label.to_owned() + " bind group")),
        });

        Self { bind_group, buffer }
    }
}