//! Mesh slicer utility for the Bevy Engine.
//!
//! TODO: Make sure the meshes generated here are working as expected with Bevy. (e.g if the
//! generated mesh is empty or too small it might cause issues)

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};

#[derive(Clone, Debug, Copy)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}

#[derive(Clone, Debug, Copy)]
struct Triangle(Vertex, Vertex, Vertex);

struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn add_triangle(&mut self, t: Triangle) {
        let base = self.vertices.len() as u32;

        // Add vertices and indices
        self.vertices.push(t.0);
        self.vertices.push(t.1);
        self.vertices.push(t.2);
        self.indices.extend([base, base + 1, base + 2]);
    }

    fn fill_boundary(&mut self, boundary: &[Vertex]) -> &Self {
        if boundary.len() < 3 {
            return self;
        }

        let center_pos =
            boundary.iter().fold(Vec3::ZERO, |acc, v| acc + v.position) / (boundary.len() as f32);
        let center = Vertex {
            position: center_pos,
            normal: Vec3::ZERO, // Will be recalculated
            uv: Vec2::ZERO,     // Placeholder
        };

        // TODO: optionally implement reordering logic if boundary isn't guaranteed CCW
        let reordered = boundary.to_vec();

        for i in (0..reordered.len()).step_by(2) {
            let a = reordered[i];
            let b = reordered[i + 1];

            let mut t = Triangle(a, b, center);

            // Recalculate Normals
            let normal = (t.1.position - t.0.position)
                .cross(t.2.position - t.0.position)
                .normalize();
            t.0.normal = normal;
            t.1.normal = normal;
            t.2.normal = normal;

            self.add_triangle(t);
        }

        self
    }

    fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    fn build(&self) -> Mesh {
        let vertices = &self.vertices;
        let indices = &self.indices;

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertices
                .iter()
                .map(|v| [v.position.x, v.position.y, v.position.z])
                .collect::<Vec<_>>(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vertices
                .iter()
                .map(|v| [v.normal.x, v.normal.y, v.normal.z])
                .collect::<Vec<_>>(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vertices
                .iter()
                .map(|v| [v.uv.x, v.uv.y])
                .collect::<Vec<_>>(),
        )
        .with_inserted_indices(Indices::U32(indices.to_vec()))
    }
}

fn edge_plane_intersection(a: Vertex, b: Vertex, plane_point: Vec3, plane_normal: Vec3) -> Vertex {
    let ab = b.position - a.position;
    let t = (plane_point - a.position).dot(plane_normal) / ab.dot(plane_normal);
    Vertex {
        position: a.position + ab * t,
        uv: a.uv + (b.uv - a.uv) * t,
        normal: (a.normal + (b.normal - a.normal) * t).normalize(),
    }
}

enum TriangleSliceResult {
    Single(Triangle),
    Split(Triangle, Triangle, Triangle),
}

fn triangle_slice(
    tri: Triangle,
    plane_normal: Vec3,
    plane_point: Vec3,
) -> (TriangleSliceResult, bool) {
    let d0 = plane_normal.dot(tri.0.position - plane_point);
    let d1 = plane_normal.dot(tri.1.position - plane_point);
    let d2 = plane_normal.dot(tri.2.position - plane_point);

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
            0 => (tri.0, tri.2, tri.1),
            1 => (tri.1, tri.0, tri.2),
            2 => (tri.2, tri.1, tri.0),
            _ => unreachable!(),
        };

        let lonely_side = sides[lonely_index];

        // Edge-plane intersections
        let first_int = edge_plane_intersection(lonely, first, plane_point, plane_normal);
        let second_int = edge_plane_intersection(lonely, second, plane_point, plane_normal);

        let single = Triangle(lonely, second_int, first_int);
        let tri1 = Triangle(first, first_int, second);
        let tri2 = Triangle(second, first_int, second_int);
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

    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
        VertexAttributeValues::Float32x3(vals) => {
            vals.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>()
        }
        _ => vec![Vec3::ZERO; positions.len()],
    };

    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(vals)) => {
            vals.iter().map(|v| Vec2::from(*v)).collect::<Vec<_>>()
        }
        _ => vec![Vec2::ZERO; positions.len()],
    };

    let triangles = match mesh.indices().unwrap() {
        Indices::U32(indices) => indices.iter().cloned().collect::<Vec<_>>(),
        Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect::<Vec<_>>(),
    }
    .chunks(3)
    .map(|c| {
        Triangle(
            Vertex {
                position: positions[c[0] as usize],
                normal: normals[c[0] as usize],
                uv: uvs[c[0] as usize],
            },
            Vertex {
                position: positions[c[1] as usize],
                normal: normals[c[1] as usize],
                uv: uvs[c[1] as usize],
            },
            Vertex {
                position: positions[c[2] as usize],
                normal: normals[c[2] as usize],
                uv: uvs[c[2] as usize],
            },
        )
    })
    .collect::<Vec<_>>();

    // Split triangles into positive / negative side
    let mut positive_mesh_builder = MeshBuilder {
        vertices: vec![],
        indices: vec![],
    };
    let mut negative_mesh_builder = MeshBuilder {
        vertices: vec![],
        indices: vec![],
    };

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
                boundary.push(single.2);
                boundary.push(single.1);

                positive_mesh_builder.add_triangle(single);
                negative_mesh_builder.add_triangle(first);
                negative_mesh_builder.add_triangle(second);
            }
            (TriangleSliceResult::Split(single, first, second), false) => {
                boundary.push(single.1);
                boundary.push(single.2);

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
        let a = Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::ZERO,
            uv: Vec2::ZERO,
        };
        let b = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
            normal: Vec3::ZERO,
            uv: Vec2::new(1.0, 0.0),
        };
        let plane_point = Vec3::new(0.5, 0.0, 0.0);
        let plane_normal = Vec3::new(1.0, 0.0, 0.0);

        // Act
        let intersection = edge_plane_intersection(a, b, plane_point, plane_normal);

        // Assert
        assert_eq!(intersection.position, Vec3::new(0.5, 0.0, 0.0));
    }

    #[test]
    fn test_triangle_slice() {
        // Arrange
        let tri = Triangle(
            Vertex {
                position: Vec3::new(0.0, 1.0, 0.0),
                normal: Vec3::ZERO,
                uv: Vec2::ZERO,
            },
            Vertex {
                position: Vec3::new(-1.0, -1.0, 0.0),
                normal: Vec3::ZERO,
                uv: Vec2::ZERO,
            },
            Vertex {
                position: Vec3::new(1.0, -1.0, 0.0),
                normal: Vec3::ZERO,
                uv: Vec2::ZERO,
            },
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
