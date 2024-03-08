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

@group(0) @binding(0)
var texture_accum: texture_2d<f32>;
@group(0) @binding(1)
var texture_reveal: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let coords: vec2i = vec2i(pos.xy);

    let reveal: f32 = textureLoad(texture_reveal, coords, 0).r;
    var accum: vec4f = textureLoad(texture_accum, coords, 0);
    return vec4(accum.rgb / max(accum.a, 0.0005), 1.0 - reveal);
}