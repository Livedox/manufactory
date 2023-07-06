
struct CameraUniform {
    proj_view: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) layer: f32,
    @location(3) light: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: f32,
    @location(2) light: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.proj_view * vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    out.layer = model.layer;
    out.light = vec4(model.light.rgb, 1.0) + model.light.a;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture = textureSample(t_diffuse, s_diffuse, in.uv, u32(in.layer));
    let ambient = vec4(0.01, 0.01, 0.01, 1.0);
    return (ambient + in.light) * texture;
}



// pub mod vs {
// 	vulkano_shaders::shader! {
//         ty: "vertex",
//         src: r"
//             #version 450

//             layout(location = 0) in vec3 position;
//             layout(location = 1) in vec2 uv;
//             layout(location = 2) in float layer;
//             layout (location = 3) in vec4 v_light;

//             layout(location = 0) out vec2 tex_coords;
//             layout(location = 1) out float f_layer;
//             layout(location = 2) out vec3 frag_pos;
//             layout(location = 3) out vec4 a_color;


//             layout (set = 1, binding = 0) uniform ProjectionView {
//                 mat4 projection_view;
//             } data;

//             void main() {
//                 a_color = vec4(v_light.rgb, 1.0) + v_light.a;

//                 gl_Position = data.projection_view * vec4(position, 1.0);
//                 tex_coords = uv;
//                 f_layer = layer;
//                 frag_pos = position;
//             }
//         ",
//     }
// }

// pub mod fs {
// 	vulkano_shaders::shader! {
//         ty: "fragment",
//         src: r"
//             #version 450

//             layout(location = 0) in vec2 tex_coords;
//             layout(location = 1) in float f_layer;
//             layout(location = 2) in vec3 frag_pos;
//             layout(location = 3) in vec4 a_color;
//             layout(location = 0) out vec4 f_color;

//             layout(set = 0, binding = 0) uniform sampler2DArray tex;
//             layout(set = 0, binding = 1) uniform Ambient_Data {
//                 vec3 color;
//                 float intensity;
//             } ambient;

//             void main() {
//                 vec3 ambient_color = ambient.intensity * ambient.color;
//                 vec4 texture = texture(tex, vec3(tex_coords, f_layer));
//                 vec4 combined_color = (a_color + vec4(ambient_color, 1.0)) * texture;
//                 f_color = combined_color;
//             }
//         ",
//     }
// }