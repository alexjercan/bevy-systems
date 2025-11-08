use bevy::prelude::*;

use super::util::TriangleMeshBuilder;

pub fn octahedron_sphere(resolution: u32) -> Mesh {
    let mut builder = TriangleMeshBuilder::default();

    // Base octahedron vertices
    let up = Vec3::Y;
    let down = -Vec3::Y;
    let left = -Vec3::X;
    let right = Vec3::X;
    let forward = Vec3::Z;
    let back = -Vec3::Z;

    let faces = [
        (up, back, left),
        (up, right, back),
        (up, forward, right),
        (up, left, forward),
        (down, left, back),
        (down, back, right),
        (down, right, forward),
        (down, forward, left),
    ];

    for (a, b, c) in faces {
        subdivide_face(&mut builder, a, b, c, resolution);
    }

    builder.build()
}

/// Linearly interpolate along the great circle (spherical interpolation)
fn slerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let dot = a.dot(b).clamp(-1.0, 1.0);
    let theta = dot.acos() * t;
    let relative = (b - a * dot).normalize();
    (a * theta.cos() + relative * theta.sin()).normalize()
}

/// Recursively subdivide a triangle face
fn subdivide_face(builder: &mut TriangleMeshBuilder, a: Vec3, b: Vec3, c: Vec3, depth: u32) {
    if depth == 0 {
        builder.add_triangle(Triangle3d::new(a, b, c));
    } else {
        let ab = slerp(a, b, 0.5);
        let bc = slerp(b, c, 0.5);
        let ca = slerp(c, a, 0.5);

        // Recursively subdivide into 4 smaller triangles
        subdivide_face(builder, a, ab, ca, depth - 1);
        subdivide_face(builder, b, bc, ab, depth - 1);
        subdivide_face(builder, c, ca, bc, depth - 1);
        subdivide_face(builder, ab, bc, ca, depth - 1);
    }
}
