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

// Fragment shader
@group(0) @binding(0)
var texture_color: texture_2d<f32>;
@group(0) @binding(1)
var reveal_color: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let reveal = textureLoad(reveal_color, vec2<i32>(pos.xy), 0);
    let color = textureLoad(texture_color, vec2<i32>(pos.xy), 0);
    let out = vec4<f32>(
        color.rgb / max(color.a, 0.00001),
        0.5);
    return out;
}