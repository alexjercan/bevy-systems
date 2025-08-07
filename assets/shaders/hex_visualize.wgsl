#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> mode: u32;
@group(2) @binding(1) var<uniform> height: f32;
@group(2) @binding(2) var<uniform> temperature: f32;
@group(2) @binding(3) var<uniform> humidity: f32;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var color: vec4<f32>;
    if (mode == 0) {
        let h = (height + 1.0) / 2.0;
        color = vec4(h, h, h, 1.0);
    } else if (mode == 1) {
        let t = (temperature + 1.0) / 2.0;
        color = mix(
            vec4(0.0, 0.0, 1.0, 1.0),
            vec4(1.0, 0.0, 0.0, 1.0),
            t
        );
    } else if (mode == 2) {
        let h = (humidity + 1.0) / 2.0;
        color = mix(
            vec4(1.0, 1.0, 0.0, 1.0),
            vec4(0.0, 1.0, 0.0, 1.0),
            h
        );
    } else {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }

    return color;
}
