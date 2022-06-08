var<private> COLORS_COUNTS: i32 = 4;

struct Config {
    dt_min: f32;
    dt_max: f32;
    dt_min_log2: f32;
    dt_max_log2: f32;
    max_width: f32;
    len: i32;
    colors: mat4x4<f32>;
    dts: vec4<f32>;
};

[[group(1), binding(0)]]
var<uniform> config: Config;

struct Frametimes {
    values: array<f32>;
};

[[group(1), binding(1)]]
var<storage> frametimes: Frametimes;

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
    if (dt < config.dts[0]) {
        return config.colors[0];
    }

    for (var i = 0; i < COLORS_COUNTS; i = i + 1) {
        if (dt < config.dts[i]) {
            let t = (dt - config.dts[i - 1]) / (config.dts[i] - config.dts[i - 1]);
            return mix(config.colors[i - 1], config.colors[i], t);
        }
    }
    return config.colors[COLORS_COUNTS - 1];
}


[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let dt_min = config.dt_min;
    let dt_max = config.dt_max;
    let dt_min_log2 = config.dt_min_log2;
    let dt_max_log2 = config.dt_max_log2;

    let max_width = config.max_width;

    var pos = in.uv.xy;

    // The general alogrithm is highly inspired by
    // <https://github.com/sawickiap/RegEngine/blob/613c31fd60558a75c5b8902529acfa425fc97b2a/Source/Game.cpp#L331>

    var width = 0.0;
    for (var i = 0; i <= config.len; i = i + 1) {
        let dt = frametimes.values[i];
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
