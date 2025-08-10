#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
}

@group(2) @binding(100) var<uniform> chunk_radius: u32;
@group(2) @binding(101) var<uniform> hex_size: f32;
@group(2) @binding(102) var<uniform> chunk_center: vec2<i32>;
@group(2) @binding(103) var<uniform> start_color: vec4<f32>;
@group(2) @binding(104) var<uniform> end_color: vec4<f32>;
@group(2) @binding(105) var<storage, read> values: array<f32>;

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
    pbr_input.material.base_color = mix(start_color, end_color, values[index]);

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

