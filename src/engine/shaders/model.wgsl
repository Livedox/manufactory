var<private> ROTATION: array<mat4x4<f32>, 4> = array(
    mat4x4f(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ),
    mat4x4f(
        0.0, 0.0, 1.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0, 1.0,
        0.0, 0.0, 0.0, 1.0,
    ),
    mat4x4f(
        -1.0, 0.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, -1.0, 1.0,
        0.0, 0.0, 0.0, 1.0,
    ),
    mat4x4f(
        0.0, 0.0, -1.0, 1.0,
        0.0, 1.0, 0.0, 0.0,
        1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
);

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
}

struct InstanceInput {
    @location(2) position: vec3<f32>,
    @location(3) light: vec4<f32>,
    @location(4) rotation_index: u32,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec4<f32>
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var position = vec4<f32>(model.position, 1.0) * ROTATION[instance.rotation_index];
    position += vec4<f32>(instance.position, 0.0);
    let light = max(sun*instance.light.a/1.7, instance.light.rgb);
    return VertexOutput(
        camera.proj_view * position,
        model.uv,
        vec4(light, 1.0));
}

// Fragment shader
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(t_diffuse, s_diffuse, in.uv);
    let ambient = vec4(0.0075, 0.0075, 0.0075, 0.0);
    return (ambient + in.light) * texture;
}