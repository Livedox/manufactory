struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: u32,
    @location(2) light: vec4<f32>,
}

@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(1)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(t_diffuse, s_diffuse, in.uv, in.layer);
    let ambient = vec4(0.0075, 0.0075, 0.0075, 0.0);
    // let ambient = vec4(1.0, 1.0, 1.0, 0.0);
    return (ambient + in.light) * texture;
}
