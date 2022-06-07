
struct FrametimeMaterial {
    dt_min: f32;
    dt_max: f32;
    dt_min_log2: f32;
    dt_max_log2: f32;
    max_width: f32;
    len: i32;
    frametimes: array<f32>;
};
[[group(1), binding(0)]]
var<storage> material: FrametimeMaterial;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

fn sdf_square(pos: vec2<f32>, half_size: vec2<f32>, offset: vec2<f32>) -> f32 {
    let p = pos - offset;
    let dist = abs(p) - half_size;
    let outside_dist = length(max(dist, vec2<f32>(0.0, 0.0)));
    let inside_dist = min(max(dist.x, dist.y), 0.0);
    return outside_dist + inside_dist;
}

fn color_from_dt(dt: f32) -> vec4<f32> {
    return mix(vec4<f32>(0., 255., 0., 1.), vec4<f32>(255., 0., 0., 1.), dt / 10.0);
}

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let dt_min = material.dt_min;
    let dt_max = material.dt_max;
    let max_width = material.max_width;
    let dt_min_log2 = material.dt_min_log2;
    let dt_max_log2 = material.dt_max_log2;

    var pos = in.uv.xy;

    var width = 0.0;
    for (var i = 0; i <= material.len; i = i + 1) {
        let dt = material.frametimes[i];
        let frame_width = (dt / dt_min);
        let frame_width = frame_width / max_width;

        let frame_height_factor = (log2(dt) - dt_min_log2) / (dt_max_log2 - dt_min_log2);
        let frame_height_factor_norm = min(max(0.0, frame_height_factor), 1.0);
        let frame_height = mix(0.0, 1.0, frame_height_factor_norm);

        if (sdf_square(pos, vec2<f32>(frame_width / 2.0, frame_height), vec2<f32>(width + frame_width / 2., 1.)) < 0.0) {
            return color_from_dt(dt);
        }

        width = width + frame_width;
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.25);
}



