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

@group(0) @binding(2)
var font_texture: texture_2d<f32>;
@group(0) @binding(3)
var font_sampler: sampler;

// numbers
let ch_0 = 48;
let ch_1 = 49;
let ch_2 = 50;
let ch_3 = 51;
let ch_4 = 52;
let ch_5 = 53;
let ch_6 = 54;
let ch_7 = 55;
let ch_8 = 56;
let ch_9 = 57;

// uppercase letters
let ch_A = 65;
let ch_B = 66;
let ch_C = 67;
let ch_D = 68;
let ch_E = 69;
let ch_F = 70;
let ch_G = 71;
let ch_H = 72;
let ch_I = 73;
let ch_J = 74;
let ch_K = 75;
let ch_L = 76;
let ch_M = 77;
let ch_N = 78;
let ch_O = 79;
let ch_P = 80;
let ch_Q = 81;
let ch_R = 82;
let ch_S = 83;
let ch_T = 84;
let ch_U = 85;
let ch_V = 86;
let ch_W = 87;
let ch_X = 88;
let ch_Y = 89;
let ch_Z = 90;
let ch_a = 97;
let ch_b = 98;
let ch_c = 99;

// lowercase letters
let ch_d = 100;
let ch_e = 101;
let ch_f = 102;
let ch_g = 103;
let ch_h = 104;
let ch_i = 105;
let ch_j = 106;
let ch_k = 107;
let ch_l = 108;
let ch_m = 109;
let ch_n = 110;
let ch_o = 111;
let ch_p = 112;
let ch_q = 113;
let ch_r = 114;
let ch_s = 115;
let ch_t = 116;
let ch_u = 117;
let ch_v = 118;
let ch_w = 119;
let ch_x = 120;
let ch_y = 121;
let ch_z = 122;

// symbols
let ch_space = 32;
let ch_dot = 46;
let ch_colon = 58;

let FONT_SIZE: f32 = 1.5;
var<private> TEXT_CURRENT_POS: vec2<f32> = vec2<f32>(0., 0.);
var<private> TEXT_OUTPUT: f32 = 0.0;

// loosely based on <https://www.shadertoy.com/view/stVBRR>
fn sdf_texture_char(p: vec2<f32>, c: i32) -> f32 {
    let char_uv = p / 16. + fract(vec2<f32>(vec2<i32>(c, c / 16)) / 16.);
    let char_sample = textureSample(font_texture, font_sampler, char_uv);
    if (p.x < 0.0 || p.x > 1. || p.y < 0.0 || p.y > 1.) {
        return 0.0;
    }
    return char_sample.x;
}

fn print(c: i32) {
    let out = sdf_texture_char(TEXT_CURRENT_POS, c);
    TEXT_CURRENT_POS.x -= .5;
    TEXT_OUTPUT += out;
}

fn newline(uv: vec2<f32>) {
    TEXT_CURRENT_POS.x = (uv.x * 64. / FONT_SIZE);
    TEXT_CURRENT_POS.y -= 1.;
}

fn get_digit(in_value: f32) -> i32 {
    var value = floor(in_value);
    if (value == 0.0) {
        return ch_0;
    }
    if (value == 1.0) {
        return ch_1;
    }
    if (value == 2.0) {
        return ch_2;
    }
    if (value == 3.0) {
        return ch_3;
    }
    if (value == 4.0) {
        return ch_4;
    }
    if (value == 5.0) {
        return ch_5;
    }
    if (value == 6.0) {
        return ch_6;
    }
    if (value == 7.0) {
        return ch_7;
    }
    if (value == 8.0) {
        return ch_8;
    }
    if (value == 9.0) {
        return ch_9;
    }
    return 0;
}

fn print_number(number: f32) {
    for (var i = 3; i >= -1; i -= 1) {
        let digit = (number / pow(10., f32(i))) % 10.;
        if (i == -1) {
            // add decimal point
            print(ch_dot);
        }
        if (abs(number) > pow(10., f32(i)) || i == 0) {
            print(get_digit(digit));
        }
    }
}

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

fn draw_frametime_graph(uv: vec2<f32>, width: f32, height: f32) -> vec4<f32> {
    // Frametime graph
    let dt_min = config.dt_min;
    let dt_max = config.dt_max;
    let dt_min_log2 = config.dt_min_log2;
    let dt_max_log2 = config.dt_max_log2;
    let max_width = config.max_width;

    // The general alogrithm is highly inspired by
    // <https://github.com/sawickiap/RegEngine/blob/613c31fd60558a75c5b8902529acfa425fc97b2a/Source/Game.cpp#L331>

    let graph_area = vec2<f32>(width, height);
    let pos_in_area = (uv * vec2<f32>(1.0, -1.0) + graph_area + vec2<f32>(0.0, height)) / graph_area;
    var graph_width = 0.0;
    for (var i = 0; i <= config.len; i = i + 1) {
        let dt = frametimes.values[i];
        let frame_width = (dt / dt_min);
        let frame_width = frame_width / max_width;

        let frame_height_factor = (log2(dt) - dt_min_log2) / (dt_max_log2 - dt_min_log2);
        let frame_height_factor_norm = min(max(0.0, frame_height_factor), 1.0);
        let frame_height = mix(0.0, 1.0, frame_height_factor_norm);

        let size = vec2<f32>(frame_width, frame_height) / 2.;
        let offset = vec2<f32>(1. + graph_width + frame_width / 2., frame_height / 2.);
        if (sdf_square(pos_in_area, size, offset) < 0.0) {
            return color_from_dt(dt);
        }

        graph_width = graph_width + frame_width;
    }
    return vec4<f32>(0.0);
}

struct VertexOutput {
    @builtin(position) uv: vec4<f32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec4<f32>(f32(in_vertex_index & 1u), f32(in_vertex_index >> 1u), 0.5, 0.5) * 4.0 - 1.0;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let background = vec4<f32>(0.0, 0.0, 0.0, 0.4);
    let area_width = 150.0;
    let section_height = 50.;
    let total_area = vec2<f32>(area_width, section_height * 2.);
    let uv = in.uv.xy / f32(textureDimensions(font_texture).y);
    if (in.uv.x > total_area.x || in.uv.y > total_area.y) {
        discard;
    }

    TEXT_CURRENT_POS = uv * 64. / FONT_SIZE;

    print_number(frametimes.fps);
    print(ch_space);
    print(ch_f);
    print(ch_p);
    print(ch_s);
    newline(uv);
    print(ch_f);
    print(ch_r);
    print(ch_a);
    print(ch_m);
    print(ch_e);
    print(ch_t);
    print(ch_i);
    print(ch_m);
    print(ch_e);
    print(ch_colon);

    var graph_color = draw_frametime_graph(in.uv.xy, area_width, section_height);
    if (any(graph_color != vec4<f32>(0.0))) {
        return graph_color;
    }

    if (TEXT_OUTPUT > 0.0) {
        return vec4<f32>(TEXT_OUTPUT);
    } else {
        return background;
    }
}
