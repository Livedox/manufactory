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