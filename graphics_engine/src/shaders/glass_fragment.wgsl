struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: u32,
    @location(2) light: vec4<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) reveal: f32,
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(1)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let ambient = vec4(0.0075, 0.0075, 0.0075, 0.0);
    let z = in.clip_position.z;
    let color = (ambient + in.light) * textureSample(t_diffuse, s_diffuse, in.uv, in.layer);
    // let weight = color.a * clamp(0.03 / (1e-5 + pow(z / 200, 4.0)), 1e-2, 3e3);
    // let weight = max(min(1.0, max(max(color.r, color.g), color.b) * color.a), color.a) *
    //     clamp((0.03 / (0.00001 + pow(z / 200.0, 4.0))), 0.01, 300.0);
    let weight = pow(color.a + 0.01, 4.0) + max(1e-2, min(3.0 * 1e3, 100.0 / (1e-5 + pow(abs(z) / 10.0, 3.0) + pow(abs(z) / 200.0, 6.0))));

    let accum = vec4f(color.rgb * color.a, color.a) * weight;

    return FragmentOutput(accum, color.a);
}