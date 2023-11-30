// WGPU dont support line width

@group(0) @binding(0)
var<uniform> u_ar_scale: vec2<f32>;

var<private> LINES: array<vec4<f32>, 12> = array(
    // horizontal
    vec4f(-0.05, -0.005, 0.0, 1.0),
    vec4f(-0.05, 0.005, 0.0, 1.0),
    vec4f(0.05, 0.005, 0.0, 1.0),

    vec4f(0.05, 0.005, 0.0, 1.0),
    vec4f(0.05, -0.005, 0.0, 1.0),
    vec4f(-0.05, -0.005, 0.0, 1.0),

    // vertical
    vec4f(0.005, 0.05, 0.0, 1.0),
    vec4f(0.005, -0.05, 0.0, 1.0),
    vec4f(-0.005, -0.05, 0.0, 1.0),

    vec4f(-0.005, -0.05, 0.0, 1.0),
    vec4f(-0.005, 0.05, 0.0, 1.0),
    vec4f(0.005, 0.05, 0.0, 1.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4<f32> {
    var pos = LINES[in_vertex_index];
    pos.x *= u_ar_scale[0] * u_ar_scale[1];
    pos.y *= u_ar_scale[1];
    return pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.8, 0.8, 0.8, 0.8);
}