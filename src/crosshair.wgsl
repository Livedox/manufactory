// WGPU dont support line width

@group(0) @binding(0)
var<uniform> u_ar: f32;
@group(1) @binding(0)
var<uniform> u_scale: f32;

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
    pos.x *= u_ar * u_scale;
    pos.y *= u_scale;
    return pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.8, 0.8, 0.8, 0.8);
}