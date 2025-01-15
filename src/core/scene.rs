use std::{cell::RefCell, default, fmt::{self, Display, Formatter}, sync::{Arc, Mutex}};

use crate::core::camera::Camera;

use glam::{Affine3A, Vec3};

use crate::geometry::{Mesh, Tri, Vertex};

use super::Color;

#[derive(Clone, Debug, Copy)]
pub enum RenderMode {
    Solid,
    Wireframe,
    FixedPoint
}

impl Display for RenderMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RenderMode::Wireframe => write!(f, "Wireframe"),
            RenderMode::FixedPoint => write!(f, "Fixed Point"),
            RenderMode::Solid => write!(f, "Standard"),
        }
    }
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub mesh: Mesh,
    pub transform: glam::Affine3A,
    render_mode: Arc<Mutex<RenderMode>>,
}

#[derive(Clone)]
// FIX: Doesnt work !!!!!!!!!!
pub struct Environment {
    pub background: Background,
    // TODO: lighting
}

#[derive(Clone)]
pub enum Background {
    Void, // Nothing
    BlenderFloor {
        size: i32,
        cell_size: f32,

        primary_color: Color,
        secondary_color: Color,
    },
    Room {
        size: i32,
        cell_size: f32,

        wall_colors: [Color; 4],
    },
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Entity: {:?}", self.name)
    }
}

impl Environment {
    pub fn new(background: Background) -> Self {
        Self { background }
    }

    pub fn get_mesh(&self) -> Option<Mesh> {
        match &self.background {
            Background::Void => None,
            Background::BlenderFloor {
                size,
                cell_size,
                primary_color,
                secondary_color,
            } => Some(Self::generate_blender_floor(
                *size,
                *cell_size,
                *primary_color,
                *secondary_color,
            )),
            Background::Room {
                size,
                cell_size,
                wall_colors,
            } => Some(Self::generate_room(*size, *cell_size, wall_colors)),
        }
    }

    // TODO: work on geting materials implemented, after that, re-impliment this function using one singular plane (4 big ass tris) and a checkered texture
    fn generate_blender_floor(
        size: i32,
        cell_size: f32,
        primary_color: Color,
        secondary_color: Color,
    ) -> Mesh {
        let mut mesh = Mesh::new();
        let grid_size = size;

        // Generate grid lines
        for i in -grid_size..=grid_size {
            // X axis lines
            mesh.vertices.push(Vertex {
                pos: Vec3::new(i as f32 * cell_size, 0.0, -grid_size as f32 * cell_size),
                uv: None,
                color: Some(primary_color),
            });
            mesh.vertices.push(Vertex {
                pos: Vec3::new(i as f32 * cell_size, 0.0, grid_size as f32 * cell_size),
                uv: None,
                color: Some(primary_color),
            });

            // Z axis lines
            mesh.vertices.push(Vertex {
                pos: Vec3::new(-grid_size as f32 * cell_size, 0.0, i as f32 * cell_size),
                uv: None,
                color: Some(primary_color),
            });
            mesh.vertices.push(Vertex {
                pos: Vec3::new(grid_size as f32 * cell_size, 0.0, i as f32 * cell_size),
                uv: None,
                color: Some(primary_color),
            });
        }

        // Add RGB axis indicators
        // X axis (red)
        mesh.vertices.extend_from_slice(&[
            Vertex {
                pos: Vec3::ZERO,
                uv: None,
                color: Some(Color::RED),
            },
            Vertex {
                pos: Vec3::new(grid_size as f32 * cell_size, 0.0, 0.0),
                uv: None,
                color: Some(Color::RED),
            },
        ]);

        // Y axis (green)
        mesh.vertices.extend_from_slice(&[
            Vertex {
                pos: Vec3::ZERO,
                uv: None,
                color: Some(Color::GREEN),
            },
            Vertex {
                pos: Vec3::new(0.0, grid_size as f32 * cell_size, 0.0),
                uv: None,
                color: Some(Color::GREEN),
            },
        ]);

        // Z axis (blue)
        mesh.vertices.extend_from_slice(&[
            Vertex {
                pos: Vec3::ZERO,
                uv: None,
                color: Some(Color::BLUE),
            },
            Vertex {
                pos: Vec3::new(0.0, 0.0, grid_size as f32 * cell_size),
                uv: None,
                color: Some(Color::BLUE),
            },
        ]);

        // Generate triangles for lines
        for i in 0..(mesh.vertices.len() / 2) {
            mesh.tris.push(Tri {
                vertices: [i * 2, i * 2 + 1, i * 2],
                normals: None,
                material: None,
            });
        }

        mesh.calculate_normals();
        mesh
    }

    fn generate_room(size: i32, cell_size: f32, wall_colors: &[Color; 4]) -> Mesh {
        let mut mesh = Mesh::new();
        let room_size = size as f32 * cell_size;
        let height = room_size * 0.8; // 80% of size for height

        // Floor vertices with checkerboard pattern
        for i in -size..=size {
            for j in -size..=size {
                let x = i as f32 * cell_size;
                let z = j as f32 * cell_size;
                let color: Color = if (i + j) % 2 == 0 {
                    wall_colors[0]
                } else {
                    wall_colors[1]
                };

                mesh.vertices.push(Vertex {
                    pos: Vec3::new(x, 0.0, z),
                    uv: None,
                    color: Some(color),
                });
            }
        }

        // Wall vertices
        let wall_points = [
            // Back wall
            (
                Vec3::new(-room_size, 0.0, room_size),
                Vec3::new(room_size, height, room_size),
                wall_colors[2],
            ),
            // Left wall
            (
                Vec3::new(-room_size, 0.0, -room_size),
                Vec3::new(-room_size, height, room_size),
                wall_colors[3],
            ),
            // Right wall
            (
                Vec3::new(room_size, 0.0, room_size),
                Vec3::new(room_size, height, -room_size),
                wall_colors[3],
            ),
        ];

        // Generate wall geometry
        for (start, end, color) in wall_points.iter() {
            let vert_start: usize = mesh.vertices.len();
            mesh.vertices.extend_from_slice(&[
                Vertex {
                    pos: *start,
                    uv: None,
                    color: Some(*color),
                },
                Vertex {
                    pos: Vec3::new(end.x, start.y, end.z),
                    uv: None,
                    color: Some(*color),
                },
                Vertex {
                    pos: *end,
                    uv: None,
                    color: Some(*color),
                },
                Vertex {
                    pos: Vec3::new(start.x, end.y, start.z),
                    uv: None,
                    color: Some(*color),
                },
            ]);

            mesh.tris.extend_from_slice(&[
                Tri {
                    vertices: [vert_start, vert_start + 1, vert_start + 2],
                    normals: None,
                    material: None,
                },
                Tri {
                    vertices: [vert_start, vert_start + 2, vert_start + 3],
                    normals: None,
                    material: None,
                },
            ]);
        }

        mesh.calculate_normals();
        mesh
    }
}
impl Entity {
    pub fn new(mesh: Mesh, transform: Affine3A, name: String) -> Self {
        Self {
            mesh,
            transform,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

    pub fn from_obj(path: &str) -> Self {
        let mesh: Mesh = Mesh::from_obj(path);
        // make name be last part of path
        let name = path.split("/").last().unwrap().to_string();

        Self {
            mesh,
            transform: Affine3A::IDENTITY,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

    pub fn from_obj_with_transform(path: &str, transform: Affine3A) -> Self {
        let mesh: Mesh = Mesh::from_obj(path);
        let name = path.split("/").last().unwrap().to_string();
        Self {
            mesh,
            transform,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

    pub fn from_obj_with_scale(path: &str, scale: f32) -> Self {
        let mesh: Mesh = Mesh::from_obj(path);
        let transform: Affine3A = Affine3A::from_scale(glam::Vec3::splat(scale));
        let name = path.split("/").last().unwrap().to_string();
        Self {
            mesh,
            transform,
            render_mode: Arc::new(Mutex::new(RenderMode::Solid)),
            name
        }
    }

   pub fn set_render_mode(&self, mode: RenderMode) {
        if let Ok(mut current_mode) = self.render_mode.lock() {
            *current_mode = mode;
        }
    }

    pub fn render_mode(&self) -> Arc<Mutex<RenderMode>> {
        Arc::clone(&self.render_mode)
    }

}



#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,
    pub entities: Vec<Entity>,
    pub environment: Environment,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            entities: Vec::new(),
            environment: Environment::new(Background::Void),
        }
    }

    pub fn new_with_background(camera: Camera, background: Background) -> Self {
        Self {
            camera,
            entities: Vec::new(),
            environment: Environment::new(background),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn spin(&mut self, entity: usize) {
        self.entities[entity].transform *= glam::Affine3A::from_rotation_y(0.03);
        self.entities[entity].transform *= glam::Affine3A::from_rotation_x(0.01);
        self.entities[entity].transform *= glam::Affine3A::from_rotation_z(0.01);
    }
}

impl Default for Scene {
    fn default() -> Self {
        let cam: Camera = Camera::new(
            Vec3::new(0.0, 2.4, -6.0),
            Vec3::new(0.0, 0.0, 1.0),
            800.0 / 600.0,
        );
        Self {
            camera: cam,
            entities: Vec::new(),
            environment: Environment::new(Background::Void),
        }
    }
}
