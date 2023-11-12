struct CameraUniform {
    proj_view: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct MatrixUniform {
    transform: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> matrix: MatrixUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) bone_id: f32,
    @location(3) bone_weight: f32,
}

struct InstanceInput {
    @location(4) position: vec3<f32>,
    @location(5) model_0: vec4<f32>,
    @location(6) model_1: vec4<f32>,
    @location(7) model_2: vec4<f32>,
    @location(8) model_3: vec4<f32>,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(instance_index) id: u32,
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_0,
        instance.model_1,
        instance.model_2,
        instance.model_3,
    );

    var total_position = vec4<f32>(model.position + instance.position, 1.0);
    if model.bone_weight > 0.0 {
        var local_position = model_matrix * vec4<f32>(model.position, 1.0);
        local_position += vec4(instance.position, 0.0);
        total_position += local_position * model.bone_weight;
    }
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = camera.proj_view * total_position;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(t_diffuse, s_diffuse, in.uv);
    let color = vec4(1.0, 1.0, 1.0, 1.0);
    let ambient = vec4(0.0075, 0.0075, 0.0075, 0.0);
    return (ambient + color) * texture;
}