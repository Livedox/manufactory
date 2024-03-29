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