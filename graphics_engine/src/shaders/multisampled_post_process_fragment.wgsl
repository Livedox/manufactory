@group(0) @binding(0)
var texture_color: texture_2d<f32>;
@group(0) @binding(1)
var texture_depth: texture_depth_multisampled_2d;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let x = i32(pos.x);
    let y = i32(pos.y);
    let my = y - 1;
    let mx = x - 1;
    let py = y + 1;
    let px = x + 1;

    let sample_count = i32(textureNumSamples(texture_depth));
    for (var i = 0; i < sample_count; i++) {
        let depth = textureLoad(texture_depth, vec2<i32>(x, y), i);
        let du = textureLoad(texture_depth, vec2<i32>(x, my), i);
        let dd = textureLoad(texture_depth, vec2<i32>(x, py), i);
        let dl = textureLoad(texture_depth, vec2<i32>(mx, y), i);
        let dr = textureLoad(texture_depth, vec2<i32>(px, y), i);
        let maxv = abs(du - dd);
        let maxh = abs(dl - dr);
        let ru = (abs(depth-du) > maxv);
        let rd = (abs(depth-dd) > maxv);
        let rl = (abs(depth-dl) > maxh);
        let rr = (abs(depth-dr) > maxh);
        // Catches more gaps. Lots of unnecessary triggers.
        if (depth > 0.9999 && (ru || rd || rl || rr)) || (ru && rd && rl && rr) {
            let cu = textureLoad(texture_color, vec2<i32>(x, my), 0);
            let cd = textureLoad(texture_color, vec2<i32>(x, py), 0);
            let cl = textureLoad(texture_color, vec2<i32>(mx, y), 0);
            let cr = textureLoad(texture_color, vec2<i32>(px, y), 0);

            // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            return mix(mix(cu, cd, 0.5), mix(cl, cr, 0.5), 0.5);
        }
    }
    

    let color = textureLoad(texture_color, vec2<i32>(pos.xy), 0);
    return vec4(color.r, color.g, color.b, color.a);
}