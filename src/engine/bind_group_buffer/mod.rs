use wgpu::util::DeviceExt;

use super::bind_group_layout::Layouts;


pub(crate) struct BindGroupsBuffers {
    pub sun: BindGroupBuffer,
    pub crosshair_aspect_scale: BindGroupBuffer,
    pub camera: BindGroupBuffer,
    pub time: BindGroupBuffer,
}

impl BindGroupsBuffers {
    pub fn new(device: &wgpu::Device, layouts: &Layouts, proj_view: &[[f32; 4]; 4]) -> Self {Self{
        sun: BindGroupBuffer::new(device, bytemuck::cast_slice(&[1.0, 1.0, 1.0]), &layouts.sun, "sun"),
        crosshair_aspect_scale: BindGroupBuffer::new(
            device, bytemuck::cast_slice(&[0.0f32, 0.0]),
            &layouts.crosshair_aspect_scale, "crosshair_aspect_scale"),
        camera: BindGroupBuffer::new(device, bytemuck::cast_slice(proj_view), &layouts.camera, "camera"),
        time: BindGroupBuffer::new(device, &0.0f32.to_be_bytes(), &layouts.time, "time"),
    }}
}


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
            layout,
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