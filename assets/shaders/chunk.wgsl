#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> chunk_radius: u32;
@group(2) @binding(1) var<uniform> hex_size: f32;
@group(2) @binding(2) var<uniform> chunk_center: vec2<i32>;
@group(2) @binding(3) var<storage, read> noise: array<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var pos = in.world_position.xz;
    {
        // Kind of meh, but it works (wish I could have used `in.world_normal`
        // here, but edges don't work with that)
        let hex = world_to_hex(pos, hex_size);
        let direction = vec2<f32>(sign(hex - chunk_center));
        pos = pos + vec2<f32>(0.1, 0.1) * (-direction);
    }

    let hex = world_to_hex(pos, hex_size);
    let hex_offset = hex - chunk_center;
    let index = hex_to_index(hex_offset, chunk_radius);

    return noise_to_color(noise[index]);
}

fn noise_to_color(noise: f32) -> vec4<f32> {
    if (noise <= -0.5) {
        return vec4<f32>(0.0, 0.0, 139.0 / 255.0, 1.0); // Deep Water
    } else if (noise <= 0.0) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0); // Water
    } else if (noise <= 0.1) {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0); // Sand
    } else if (noise <= 0.3) {
        return vec4<f32>(0.0, 128.0 / 255.0, 0.0, 1.0); // Grass
    } else if (noise <= 0.6) {
        return vec4<f32>(139.0 / 255.0, 69.0 / 255.0, 19.0 / 255.0, 1.0); // Hills
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // Mountains
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
    let r_clamped = clamp(hex.y, -i32(radius), i32(radius));

    let q_offset = q_clamped + i32(radius);
    let r_offset = r_clamped + i32(radius);
    return u32(clamp(r_offset * i32(size) + q_offset, 0, i32(size * size) - 1));
}
