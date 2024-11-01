use glam::Vec3;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub verts: Vec<Vert>,
    pub indices: Vec<u32>,
    norms_dirty: bool,
}

impl Mesh {
    pub fn new(verts: Vec<Vert>, indices: Vec<u32>) -> Self {
        Mesh {
            verts,
            indices,
            norms_dirty: true,
        }
    }

    pub fn create_cube() -> Self {
        let verts = vec![
            Vert {
                pos: Vec3::new(-1.0, -1.0, -1.0),
                ..Vert::default()
            },
            Vert {
                pos: Vec3::new(1.0, -1.0, -1.0),
                ..Vert::default()
            },
            Vert {
                pos: Vec3::new(1.0, 1.0, -1.0),
                ..Vert::default()
            },
            Vert {
                pos: Vec3::new(-1.0, 1.0, -1.0),
                ..Vert::default()
            },
            Vert {
                pos: Vec3::new(-1.0, -1.0, 1.0),
                ..Vert::default()
            },
            Vert {
                pos: Vec3::new(1.0, -1.0, 1.0),
                ..Vert::default()
            },
            Vert {
                pos: Vec3::new(1.0, 1.0, 1.0),
                ..Vert::default()
            },
            Vert {
                pos: Vec3::new(-1.0, 1.0, 1.0),
                ..Vert::default()
            },
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front
            1, 5, 6, 6, 2, 1, // Right
            7, 6, 5, 5, 4, 7, // Back
            4, 0, 3, 3, 7, 4, // Left
            4, 5, 1, 1, 0, 4, // Bottom
            3, 2, 6, 6, 7, 3, // Top
        ];

        Mesh::new(verts, indices)
    }

}


#[derive(Clone, Copy, Debug)]
pub struct Vert {
    pub pos: Vec3,
    pub norm: Vec3,
    pub color: crate::core::color::Color,
}

impl Default for Vert {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            norm: Vec3::ZERO,
            color: crate::core::color::Color::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tri {
    pub indices: [u32; 3],
    pub norm: Vec3,
    pub centroid: Vec3,
    /// Min `bounds.0` and Max `bounds.1`
    pub bounds: (Vec3, Vec3),
}

impl Tri {
    pub fn new(idx: [u32; 3], verts: &[Vert]) -> Self {
        let v0 = verts[idx[0] as usize].pos;
        let v1 = verts[idx[1] as usize].pos;
        let v2 = verts[idx[2] as usize].pos;

        let norm = (v1 - v0).cross(v2 - v0).normalize();
        let centroid = (v0 + v1 + v2) / 3.0;

        let min_bound: Vec3 = v0.min(v1).min(v2);
        let max_bound: Vec3 = v0.max(v1).max(v2);

        Tri {
            indices: idx,
            norm,
            centroid,
            bounds: (min_bound, max_bound),
        }
    }
}
