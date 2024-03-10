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