struct CameraUniform {
    proj_view: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(
    line: VertexInput,
) -> @builtin(position) vec4<f32> {
    var position = camera.proj_view*vec4<f32>(line.position, 1.0);
    position.z -= 0.0001;
    return position;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.0, 0.0, 0.0, 1.0);
}