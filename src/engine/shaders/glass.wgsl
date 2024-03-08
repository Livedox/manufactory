@group(0) @binding(0)
var<uniform> sun: vec3<f32>;

struct CameraUniform {
    proj_view: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) layer: u32,
    @location(3) light: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: u32,
    @location(2) light: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let light = max(sun*model.light.a/1.7, model.light.rgb);
    var position = camera.proj_view*vec4<f32>(model.position, 1.0);
    return VertexOutput(
        position,
        model.uv,
        model.layer,
        vec4(light, 1.0));
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(1)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient = vec4(0.0075, 0.0075, 0.0075, 0.0);
    let z = in.clip_position.z;
    let color = (ambient + in.light) * textureSample(t_diffuse, s_diffuse, in.uv, in.layer);
    // let weight = max(min(1.0, max(max(color.r, color.g), color.b) * color.a), color.a) *
    //     clamp((0.03 / (0.00001 + pow(z / 200.0, 4.0))), 0.01, 300.0);
    let weight = 1.0;
    let exit = vec4f(color.rgb * weight, color.a);

    return exit;
}