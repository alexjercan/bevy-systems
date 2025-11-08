use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub mod prelude {
    pub use super::TriangleMeshBuilder;
}

#[derive(Clone, Debug, Default)]
pub struct TriangleMeshBuilder {
    pub triangles: Vec<Triangle3d>,
}

impl TriangleMeshBuilder {
    pub fn add_triangle(&mut self, t: Triangle3d) {
        self.triangles.push(t);
    }

    pub fn vertices_and_indices(&self) -> (Vec<Vec3>, Vec<u32>) {
        let mut base = 0;
        let mut vertices = vec![];
        let mut indices = vec![];

        for t in &self.triangles {
            vertices.push(t.vertices[0]);
            vertices.push(t.vertices[1]);
            vertices.push(t.vertices[2]);

            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);

            base += 3;
        }

        (vertices, indices)
    }

    pub fn normals(&self) -> Vec<Vec3> {
        let mut normals = vec![];

        for t in &self.triangles {
            let normal = t.normal().unwrap_or(Dir3::Y).into();

            normals.push(normal);
            normals.push(normal);
            normals.push(normal);
        }

        normals
    }

    pub fn uvs(&self) -> Vec<Vec2> {
        let mut uvs = vec![];

        for t in &self.triangles {
            let a = t.vertices[0];
            let b = t.vertices[1];
            let c = t.vertices[2];

            let u_axis = (b - a).normalize();
            let v_axis = t.normal().unwrap_or(Dir3::Y).cross(u_axis).normalize();

            for v in [a, b, c] {
                let local = v - a;
                uvs.push(Vec2::new(local.dot(u_axis), local.dot(v_axis)));
            }
        }

        uvs
    }

    pub fn fill_boundary(&mut self, boundary: &[Vec3]) -> &Self {
        if boundary.len() < 3 {
            return self;
        }

        let center = boundary.iter().fold(Vec3::ZERO, |acc, v| acc + v) / (boundary.len() as f32);

        // TODO: optionally implement reordering logic if boundary isn't guaranteed CCW
        let reordered = boundary.to_vec();

        for i in (0..reordered.len()).step_by(2) {
            let a = reordered[i];
            let b = reordered[i + 1];

            let t = Triangle3d::new(a, b, center);

            self.add_triangle(t);
        }

        self
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }
}

impl MeshBuilder for TriangleMeshBuilder {
    fn build(&self) -> Mesh {
        let (vertices, indices) = self.vertices_and_indices();
        let normals = self.normals();
        let uvs = self.uvs();

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertices.iter().map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            normals.iter().map(|n| [n.x, n.y, n.z]).collect::<Vec<_>>(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            uvs.iter().map(|u| [u.x, u.y]).collect::<Vec<_>>(),
        )
        .with_inserted_indices(Indices::U32(indices.to_vec()))
    }
}
