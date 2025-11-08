//! Mesh slicer utility for the Bevy Engine.
//!
//! TODO: Make sure the meshes generated here are working as expected with Bevy. (e.g if the
//! generated mesh is empty or too small it might cause issues)

use bevy::{
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
};

use super::util::TriangleMeshBuilder;

fn edge_plane_intersection(a: Vec3, b: Vec3, plane_point: Vec3, plane_normal: Vec3) -> Vec3 {
    let ab = b - a;
    let t = (plane_point - a).dot(plane_normal) / ab.dot(plane_normal);

    a + ab * t
}

enum TriangleSliceResult {
    Single(Triangle3d),
    Split(Triangle3d, Triangle3d, Triangle3d),
}

fn triangle_slice(
    tri: Triangle3d,
    plane_normal: Vec3,
    plane_point: Vec3,
) -> (TriangleSliceResult, bool) {
    let d0 = plane_normal.dot(tri.vertices[0] - plane_point);
    let d1 = plane_normal.dot(tri.vertices[1] - plane_point);
    let d2 = plane_normal.dot(tri.vertices[2] - plane_point);

    let sides = [d0 >= 0.0, d1 >= 0.0, d2 >= 0.0];

    // Fully positive
    if sides[0] && sides[1] && sides[2] {
        (TriangleSliceResult::Single(tri), true)
    }
    // Fully negative
    else if !sides[0] && !sides[1] && !sides[2] {
        (TriangleSliceResult::Single(tri), false)
    } else {
        // Find lonely point index
        let lonely_index = if sides[0] == sides[1] {
            2
        } else if sides[0] == sides[2] {
            1
        } else {
            0
        };

        let (lonely, first, second) = match lonely_index {
            0 => (tri.vertices[0], tri.vertices[2], tri.vertices[1]),
            1 => (tri.vertices[1], tri.vertices[0], tri.vertices[2]),
            2 => (tri.vertices[2], tri.vertices[1], tri.vertices[0]),
            _ => unreachable!(),
        };

        let lonely_side = sides[lonely_index];

        // Edge-plane intersections
        let first_int = edge_plane_intersection(lonely, first, plane_point, plane_normal);
        let second_int = edge_plane_intersection(lonely, second, plane_point, plane_normal);

        let single = Triangle3d::new(lonely, second_int, first_int);
        let tri1 = Triangle3d::new(first, first_int, second);
        let tri2 = Triangle3d::new(second, first_int, second_int);
        (TriangleSliceResult::Split(single, tri1, tri2), lonely_side)
    }
}

/// Slices a mesh along a plane defined by a normal and a point on the plane.
/// Returns two meshes: one on the positive side of the plane and one on the negative side.
pub fn mesh_slice(mesh: &Mesh, plane_normal: Vec3, plane_point: Vec3) -> Option<(Mesh, Mesh)> {
    // Extract positions, normals, uvs
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
        VertexAttributeValues::Float32x3(vals) => {
            vals.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>()
        }
        _ => panic!("Unsupported position format"),
    };

    // let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
    //     VertexAttributeValues::Float32x3(vals) => {
    //         vals.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>()
    //     }
    //     _ => vec![Vec3::ZERO; positions.len()],
    // };

    // let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
    //     Some(VertexAttributeValues::Float32x2(vals)) => {
    //         vals.iter().map(|v| Vec2::from(*v)).collect::<Vec<_>>()
    //     }
    //     _ => vec![Vec2::ZERO; positions.len()],
    // };

    let triangles = match mesh.indices().unwrap() {
        Indices::U32(indices) => indices.to_vec(),
        Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect::<Vec<_>>(),
    }
    .chunks(3)
    .map(|c| {
        Triangle3d::new(
            positions[c[0] as usize],
            positions[c[1] as usize],
            positions[c[2] as usize],
        )
    })
    .collect::<Vec<_>>();

    // Split triangles into positive / negative side
    let mut positive_mesh_builder = TriangleMeshBuilder::default();
    let mut negative_mesh_builder = TriangleMeshBuilder::default();

    let mut boundary = vec![];
    for tri in triangles {
        match triangle_slice(tri, plane_normal, plane_point) {
            (TriangleSliceResult::Single(tri), true) => {
                positive_mesh_builder.add_triangle(tri);
            }
            (TriangleSliceResult::Single(tri), false) => {
                negative_mesh_builder.add_triangle(tri);
            }
            (TriangleSliceResult::Split(single, first, second), true) => {
                boundary.push(single.vertices[2]);
                boundary.push(single.vertices[1]);

                positive_mesh_builder.add_triangle(single);
                negative_mesh_builder.add_triangle(first);
                negative_mesh_builder.add_triangle(second);
            }
            (TriangleSliceResult::Split(single, first, second), false) => {
                boundary.push(single.vertices[1]);
                boundary.push(single.vertices[2]);

                negative_mesh_builder.add_triangle(single);
                positive_mesh_builder.add_triangle(first);
                positive_mesh_builder.add_triangle(second);
            }
        }
    }

    positive_mesh_builder.fill_boundary(&boundary);
    negative_mesh_builder.fill_boundary(&boundary.iter().rev().cloned().collect::<Vec<_>>());

    if positive_mesh_builder.is_empty() || negative_mesh_builder.is_empty() {
        return None;
    }

    Some((positive_mesh_builder.build(), negative_mesh_builder.build()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_edge_plane_intersection() {
        // Arrange
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let plane_point = Vec3::new(0.5, 0.0, 0.0);
        let plane_normal = Vec3::new(1.0, 0.0, 0.0);

        // Act
        let intersection = edge_plane_intersection(a, b, plane_point, plane_normal);

        // Assert
        assert_eq!(intersection, Vec3::new(0.5, 0.0, 0.0));
    }

    #[test]
    fn test_triangle_slice() {
        // Arrange
        let tri = Triangle3d::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
        );
        let plane_point = Vec3::new(0.0, 0.0, 0.0);
        let plane_normal = Vec3::new(0.0, 1.0, 0.0);

        // Act
        let (result, is_positive) = triangle_slice(tri, plane_normal, plane_point);

        // Assert
        match result {
            TriangleSliceResult::Split(_, _, _) => assert!(true),
            _ => assert!(false, "Expected triangle to be split"),
        }
        assert!(is_positive, "Expected lonely vertex to be on positive side");
    }
}
