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
var texture_color: texture_2d<f32>;
@group(0) @binding(1)
var glass_color: texture_2d<f32>;
@group(0) @binding(2)
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


    // Catches almost all gaps. Lots of unnecessary triggers.
    // let r1 = (abs(depth-du) > maxh) && (abs(depth-dd) > maxh);
    // let r2 = (abs(depth-dl) > maxv) && (abs(depth-dr) > maxv);
    // if (depth > 0.99999 && (r1 || r2)) || (r1 && r2) {}

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

    let glass = textureLoad(glass_color, vec2<i32>(pos.xy), 0);
    let color = textureLoad(texture_color, vec2<i32>(pos.xy), 0);
    let return_color = mix(color.rgb, glass.rgb, glass.a);
    return color;
}