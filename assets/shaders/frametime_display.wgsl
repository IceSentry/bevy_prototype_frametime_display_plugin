var<private> COLORS_COUNT: i32 = 4;

struct Config {
    dt_min: f32,
    dt_max: f32,
    dt_min_log2: f32,
    dt_max_log2: f32,
    max_width: f32,
    len: i32,
    colors: mat4x4<f32>,
    dts: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> config: Config;

struct Frametimes {
    fps: f32,
    values: array<f32>,
}
@group(0) @binding(1)
var<storage> frametimes: Frametimes;

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

    for (var i = 0; i < COLORS_COUNT; i = i + 1) {
        if (dt < config.dts[i]) {
            let t = (dt - config.dts[i - 1]) / (config.dts[i] - config.dts[i - 1]);
            return mix(config.colors[i - 1], config.colors[i], t);
        }
    }
    return config.colors[COLORS_COUNT - 1];
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(f32(in_vertex_index & 1u), f32(in_vertex_index >> 1u), 0.5, 0.5) * 4.0 - 1.0;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let background = vec4<f32>(0.0, 0.0, 0.0, 0.4);
    let area_width = 150.0;
    let section_height = 50.0;
    let total_area = vec2<f32>(area_width, section_height);

    if (in.clip_position.x > total_area.x || in.clip_position.y > total_area.y) {
        discard;
    }

    let dt_min = config.dt_min;
    let dt_max = config.dt_max;
    let dt_min_log2 = config.dt_min_log2;
    let dt_max_log2 = config.dt_max_log2;
    let max_width = config.max_width;

    // The general alogrithm is highly inspired by
    // <https://github.com/sawickiap/RegEngine/blob/613c31fd60558a75c5b8902529acfa425fc97b2a/Source/Game.cpp#L331>

    let graph_area = vec2<f32>(area_width, section_height);
    let pos_in_area = (in.clip_position.xy * vec2<f32>(1.0, -1.0) + graph_area) / graph_area;
    var width = 0.0;
    for (var i = 0; i <= config.len; i = i + 1) {
        let dt = frametimes.values[i];
        let frame_width = (dt / dt_min);
        let frame_width = frame_width / max_width;

        let frame_height_factor = (log2(dt) - dt_min_log2) / (dt_max_log2 - dt_min_log2);
        let frame_height_factor_norm = min(max(0.0, frame_height_factor), 1.0);
        let frame_height = mix(0.0, 1.0, frame_height_factor_norm);

        let size = vec2<f32>(frame_width, frame_height) / 2.;
        let offset = vec2<f32>(1. + width + frame_width / 2., frame_height / 2.);
        if (sdf_square(pos_in_area, size, offset) < 0.0) {
            return color_from_dt(dt);
        }

        width = width + frame_width;
    }

    return background;
}
