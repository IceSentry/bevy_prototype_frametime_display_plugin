struct OverlayConfig {
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
var<uniform> config: OverlayConfig;

struct Frametimes {
    fps: f32,
    frame_count: u32,
    resolution: vec2<u32>,
    scale: f32,
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
let ch_lparen = 40;
let ch_rparen = 41;
let ch_percent = 37;

let FONT_SIZE: f32 = 1.3;
var<private> TEXT_CURRENT_POS: vec2<f32> = vec2<f32>(0., 0.);
var<private> TEXT_OUTPUT: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
var<private> ROW_COUNT: f32 = 0.0;

// loosely based on <https://www.shadertoy.com/view/stVBRR>
fn sdf_texture_char(pos: vec2<f32>, char_id: i32) -> vec4<f32> {
    let char_uv = pos / 16. + fract(vec2<f32>(f32(char_id), f32(char_id / 16)) / 16.);
    let char_sample = textureSample(font_texture, font_sampler, char_uv);
    if (pos.x < 0.0 || pos.x > 1. || pos.y < 0.0 || pos.y > 1.) {
        return vec4<f32>(0.0);
    }
    return char_sample;
}

// Prints the given character at the current cursor position
fn print(c: i32) {
    let out = sdf_texture_char(TEXT_CURRENT_POS, c);
    TEXT_CURRENT_POS.x -= .5;
    TEXT_OUTPUT += out;
}

// Moves the cursor to the next line
fn newline(uv: vec2<f32>) {
    TEXT_CURRENT_POS.x = (uv.x * 64. / FONT_SIZE);
    TEXT_CURRENT_POS.y -= 1.;
    ROW_COUNT += 1.0;
}

// Gets the charcode for the given number
// Only works with digits, otherwise returns 0
fn get_digit(in_value: f32) -> i32 {
    var value = floor(in_value);
    if (value >= 0.0 && value <= 9.0) {
        return 48 + i32(value);
    }
    return 0;
}

// Prints 4 numbers before the decimal point and 2 after
fn print_number(number: f32) {
    for (var i = 4; i >= -2; i -= 1) {
        // get the digit at the current index
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

// prints u32 values with up to 8 digits
fn print_u32(in_number: u32) {
    var number = f32(in_number);
    for (var i = 8; i >= 0; i -= 1) {
        // get the digit at the current index
        let digit = (number / pow(10., f32(i))) % 10.;
        if (abs(number) > pow(10., f32(i))) {
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

// Gets a color based on the delta time
// The colors are configured using the OverlayConfig
fn color_from_dt(dt: f32) -> vec4<f32> {
    if (dt < config.dts[0]) {
        return config.colors[0];
    }
    let colors_count = 4;
    for (var i = 0; i < colors_count; i = i + 1) {
        if (dt < config.dts[i]) {
            let t = (dt - config.dts[i - 1]) / (config.dts[i] - config.dts[i - 1]);
            return mix(config.colors[i - 1], config.colors[i], t);
        }
    }
    return config.colors[colors_count - 1];
}

fn draw_frametime_graph(uv: vec2<f32>, width: f32, height: f32, offset: f32) -> vec4<f32> {
    // Frametime graph
    let dt_min = config.dt_min;
    let dt_max = config.dt_max;
    let dt_min_log2 = config.dt_min_log2;
    let dt_max_log2 = config.dt_max_log2;
    let max_width = config.max_width;

    // The general alogrithm is highly inspired by
    // <https://asawicki.info/news_1758_an_idea_for_visualization_of_frame_times>
    // <https://github.com/sawickiap/RegEngine/blob/613c31fd60558a75c5b8902529acfa425fc97b2a/Source/Game.cpp#L331>

    let graph_area = vec2<f32>(width, height);
    let pos_in_area = (uv * vec2<f32>(1.0, -1.0) + graph_area + vec2<f32>(0.0, offset)) / graph_area;
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
    // fullscreen triangle
    out.uv = vec4<f32>(f32(in_vertex_index & 1u), f32(in_vertex_index >> 1u), 0.5, 0.5) * 4.0 - 1.0;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let background = vec4<f32>(0.0, 0.0, 0.0, 0.4);
    let area_width = 175.0;
    let row_height = FONT_SIZE * 16.;
    let graph_height = row_height;
    let total_area = vec2<f32>(area_width, row_height * 5.);
    let font_uv = in.uv.xy / f32(textureDimensions(font_texture).y);
    if (in.uv.x > total_area.x || in.uv.y > total_area.y) {
        discard;
    }

    TEXT_CURRENT_POS = font_uv * 64. / FONT_SIZE;

    // fps
    print_number(frametimes.fps);
    print(ch_space);
    print(ch_f);
    print(ch_p);
    print(ch_s);
    newline(font_uv);

    // frametime in ms
    let dt = frametimes.values[config.len - 1] * 1000.;
    print_number(dt);
    print(ch_m);
    print(ch_s);
    newline(font_uv);

    // frame count since start
    print(ch_F);
    print(ch_r);
    print(ch_a);
    print(ch_m);
    print(ch_e);
    print(ch_colon);
    print(ch_space);
    print_u32(frametimes.frame_count);
    newline(font_uv);

    // resolution and scale
    print_u32(frametimes.resolution.x);
    print(ch_x);
    print_u32(frametimes.resolution.y);
    print(ch_space);
    print(ch_lparen);
    print_u32(u32(frametimes.scale * 100.));
    print(ch_percent);
    print(ch_rparen);
    newline(font_uv);

    //frametime graph
    var graph_color = draw_frametime_graph(in.uv.xy, area_width, graph_height, ROW_COUNT * row_height);
    if (any(graph_color != vec4<f32>(0.0))) {
        return graph_color;
    }
    newline(font_uv);

    if (TEXT_OUTPUT.x > 0.0) {
        return TEXT_OUTPUT.xxxx;
    } else {
        return background;
    }
}
