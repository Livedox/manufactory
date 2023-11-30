const MAX_WEIGHT: u32 = 3u;

@group(0) @binding(0)
var<uniform> sun: vec3<f32>;

struct CameraUniform {
    proj_view: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> camera: CameraUniform;

@group(3) @binding(0)
var<storage, read> matricies: array<mat4x4<f32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) bone_id: vec3<i32>,
    @location(3) bone_weight: vec3<f32>,
}

struct InstanceInput {
    @location(4) position: vec3<f32>,
    @location(5) light: vec4<f32>,
    @location(6) start_matrix: u32,
    @location(7) rotation_matrix_index: u32,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec4<f32>,
}


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

@vertex
fn vs_main(
    @builtin(instance_index) id: u32,
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var total_position: vec4<f32>;
    if model.bone_id[0] != -1 {
        total_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        for (var i = 0u; i < MAX_WEIGHT; i++) {
            if model.bone_id[i] < 0 {break;}
            let local_position = matricies[instance.start_matrix+u32(model.bone_id[i])]*vec4<f32>(model.position, 1.0);
            total_position += local_position * model.bone_weight[i];
        }
    } else {
        total_position = vec4<f32>(model.position, 1.0);
    }
    total_position *= ROTATION[instance.rotation_matrix_index];
    total_position += vec4f(instance.position, 0.0);
    
    let light = max(sun*instance.light.a/1.7, instance.light.rgb);
    return VertexOutput(
        camera.proj_view * total_position,
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