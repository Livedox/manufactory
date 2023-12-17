var<private> FULLSCREEN: array<vec4<f32>, 3> = array(
    vec4<f32>(-1.0, -1.0, 0.0, 1.0),
    vec4<f32>(-1.0, 3.0, 0.0, 1.0),
    vec4<f32>(3.0, -1.0, 0.0, 1.0)
);

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Generate a triangle to fill the screen.
    // The approach is based on: https://stackoverflow.com/a/59739538/4593433
    // https://github.com/FrankenApps/wpp/blob/master/src/grayscale/shader/grayscale.wgsl
    return FULLSCREEN[in_vertex_index];
}

var<private> EPSILON: f32 = 1.0e-7;
@group(0) @binding(0)
var texture_color: texture_2d<f32>;
@group(0) @binding(1)
var texture_depth: texture_depth_2d;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let x = i32(pos.x);
    let y = i32(pos.y);
    let my = y - 1;
    let mx = x - 1;
    let py = y + 1;
    let px = x + 1;
    let depth = textureLoad(texture_depth, vec2<i32>(x, y), 0);
    let du = textureLoad(texture_depth, vec2<i32>(x, my), 0);
    let dd = textureLoad(texture_depth, vec2<i32>(x, py), 0);
    let dl = textureLoad(texture_depth, vec2<i32>(mx, y), 0);
    let dr = textureLoad(texture_depth, vec2<i32>(px, y), 0);
    let maximum_difference = max(abs(du - dd), abs(dl - dr));
    if (abs(depth-du) > maximum_difference) &&
        (abs(depth-dd) > maximum_difference) &&
        (abs(depth-dl) > maximum_difference) &&
        (abs(depth-dr) > maximum_difference) {
        return vec4(1.0, 0.0, 0.0, 1.0);
    }

    let color = textureLoad(texture_color, vec2<i32>(pos.xy), 0);
    return vec4(color.r, color.g, color.b, color.a);
}