#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
}

@group(2) @binding(100) var<uniform> chunk_radius: u32;
@group(2) @binding(101) var<uniform> hex_size: f32;
@group(2) @binding(102) var<uniform> chunk_center: vec2<i32>;
@group(2) @binding(103) var<storage, read> tiles: array<i32>;

const DEEP_WATER: i32 = 0;
const WATER: i32 = 1;
const DESERT: i32 = 2;
const GRASSLAND: i32 = 3;
const HILLS: i32 = 4;
const MOUNTAINS: i32 = 5;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // calculate the hex color
    var pos = in.world_position.xz;
    let N = normalize(in.world_normal.xyz);
    let is_vertical = abs(N.y) < 0.9;
    if is_vertical {
        pos = pos - vec2<f32>(0.1, 0.1) * sign(N.xz);
    }

    let hex = world_to_hex(pos, hex_size);
    let hex_offset = hex - chunk_center;
    let index = hex_to_index(hex_offset, chunk_radius);
    let kind = tiles[index];
    pbr_input.material.base_color = tile_kind_to_color(kind);

    if (kind == DEEP_WATER || kind == WATER) {
        let time = globals.time;
        let uv = in.uv * 5.0;
        let pos = in.world_position.xy;

        let offset = hash(floor(pos));
        let n = noise(uv + vec2(time * 0.5 + offset, time * 0.3 + offset));
        let n2 = noise(uv * 2.0 + vec2(-time * 0.7, time * 0.4) + offset);
        let ripple = (n + 0.5 * n2) * 0.05;

        pbr_input.material.base_color += ripple;
    }

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}

// 2D value noise helper
fn hash(p: vec2<f32>) -> f32 {
    // A simple but effective pseudo-random hash function
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Four corners in 2D of our cell
    let a = hash(i);
    let b = hash(i + vec2(1.0, 0.0));
    let c = hash(i + vec2(0.0, 1.0));
    let d = hash(i + vec2(1.0, 1.0));

    // Smooth interpolation (fade function)
    let u = f * f * (3.0 - 2.0 * f);

    // Bilinear interpolate the four corners
    return mix(a, b, u.x) +
           (c - a)* u.y * (1.0 - u.x) +
           (d - b) * u.x * u.y;
}

fn tile_kind_to_color(kind: i32) -> vec4<f32> {
    if (kind == DEEP_WATER) {
        return vec4<f32>(0.0, 0.18, 0.35, 1.0);
    } else if (kind == WATER) {
        return vec4<f32>(0.0, 0.3, 0.5, 1.0);
    } else if (kind == DESERT) {
        return vec4<f32>(0.85, 0.73, 0.5, 1.0);
    } else if (kind == GRASSLAND) {
        return vec4<f32>(0.4, 0.65, 0.3, 1.0);
    } else if (kind == HILLS) {
        return vec4<f32>(0.45, 0.4, 0.35, 1.0);
    } else if (kind == MOUNTAINS) {
        return vec4<f32>(0.45, 0.45, 0.45, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}

fn world_to_hex(position: vec2<f32>, size: f32) -> vec2<i32> {
    // invert the scaling
    let x = position.x / size;
    let y = position.y / size;
    // cartesian to hex
    let q = (2.0 / 3.0) * x;
    let r = (-1.0 / 3.0) * x + (sqrt(3.0) / 3.0) * y;
    // round to nearest hex
    let hex = axial_round(vec2<f32>(q, r));

    return hex;
}

fn axial_round(frac: vec2<f32>) -> vec2<i32> {
    let x = frac.x;
    let y = frac.y;
    let z = -x - y;

    var q = round(x);
    var r = round(y);
    var s = round(z);

    var q_diff = abs(q - x);
    var r_diff = abs(r - y);
    var s_diff = abs(s - z);

    if (q_diff > r_diff && q_diff > s_diff) {
        q = -r - s;
    } else if (r_diff > s_diff) {
        r = -q - s;
    } else {
        s = -q - r;
    }

    return vec2<i32>(i32(round(q)), i32(round(r)));
}

fn hex_to_index(hex: vec2<i32>, radius: u32) -> u32 {
    let size = radius * 2 + 1u;

    let q_clamped = clamp(hex.x, -i32(radius), i32(radius));
    let s = -q_clamped - hex.y;
    let s_clamped = clamp(s, -i32(radius), i32(radius));
    let r = -q_clamped - s_clamped;
    let r_clamped = clamp(r, -i32(radius), i32(radius));

    let q_offset = q_clamped + i32(radius);
    let r_offset = r_clamped + i32(radius);
    return u32(clamp(r_offset * i32(size) + q_offset, 0, i32(size * size) - 1));
}
