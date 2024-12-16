use glam::Vec3;

use crate::core::geometry::mesh::{Mesh, Normal, Tri};

pub fn compute_normal(tri: &Tri, mesh: &Mesh) -> Vec3 {
    let v0 = mesh.vertices[tri.vertices[0]].pos;
    let v1 = mesh.vertices[tri.vertices[1]].pos;
    let v2 = mesh.vertices[tri.vertices[2]].pos;

    (v1 - v0).cross(v2 - v0).normalize()
}

pub fn compute_normals(mesh: &mut Mesh) {
    mesh.normals.clear();
    mesh.normals.resize(mesh.vertices.len(), Normal { norm: Vec3::ZERO });

    for tri in &mesh.tris {
        let normal = compute_normal(tri, mesh);

        for i in 0..3 {
            mesh.normals[tri.vertices[i]].norm += normal
        }
    }

    for normal in &mut mesh.normals {
        normal.norm = normal.norm.normalize();
    }
}