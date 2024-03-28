use wgpu::util::DeviceExt;

pub fn render_selection(device: &wgpu::Device, min: &[f32; 3], max: &[f32; 3]) -> wgpu::Buffer {
    let mut v = [[0.0; 3]; 8];
    for (i, item) in v.iter_mut().enumerate() {
        *item = [
            if i&0b100 > 0 {max[0]} else {min[0]},
            if i&0b010 > 0 {max[1]} else {min[1]},
            if i&0b001 > 0 {max[2]} else {min[2]},
        ]
    }

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Selection vertex Buffer"),
        contents: bytemuck::cast_slice(&[
            v[0], v[1],   v[0], v[2],   v[0], v[4],
            v[7], v[3],   v[7], v[5],   v[7], v[6],
            v[1], v[3],   v[1], v[5],
            v[2], v[3],   v[2], v[6],
            v[4], v[5],   v[4], v[6]
        ]),
        usage: wgpu::BufferUsages::VERTEX,
    })
}