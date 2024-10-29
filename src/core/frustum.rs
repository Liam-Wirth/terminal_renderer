use glam::{Vec3, Vec4};

use super::camera::Camera;

#[derive(Debug, Default, Clone, Copy)]
pub struct FrustumPlane {
    normal: Vec3,  // Normal vector of the plane
    distance: f32, // Distance from the origin along the normal
}

impl FrustumPlane {
    /// Constructs a FrustumPlane from three points in 3D space.
    pub fn from_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let normal = (p2 - p1).cross(p3 - p1).normalize();
        let distance = -normal.dot(p1);
        FrustumPlane { normal, distance }
    }

    /// Returns the signed distance of a point from the plane.
    pub fn distance_from_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    /// Checks if a point is in front of the plane.
    pub fn is_point_in_front(&self, point: Vec3) -> bool {
        self.distance_from_point(point) >= 0.0
    }

    /// Checks if a bounding sphere (defined by its center and radius) is in front of the plane.
    /// Useful for frustum culling.
    pub fn is_sphere_in_front(&self, center: Vec3, radius: f32) -> bool {
        self.distance_from_point(center) >= -radius
    }

    fn default() -> FrustumPlane {
        FrustumPlane {
            normal: Vec3::ZERO,
            distance: 0.0,
        }
    }
}

enum FrustumPlanes {
    Left,
    Right,
    Bottom,
    Top,
    Near,
    Far,
}

const PLANE_COUNT: usize = 6;
const POINT_COUNT: usize = 8;

#[derive(Debug, Default, Clone, Copy)]
pub struct Frustum {
    pub planes: [FrustumPlane; PLANE_COUNT],
    pub points: [Vec3; POINT_COUNT],
}

impl Frustum {
    fn new() -> Self {
        Self {
            planes: [FrustumPlane::default(); PLANE_COUNT],
            points: [Vec3::ZERO; POINT_COUNT],
        }
    }

    /// Initializes the frustum planes and points based on the camera properties
    fn initialize(cam: &Camera) -> Self {
        let fov_tan = (cam.fov * 0.5).tan();
        let aspect_ratio = cam.aspect_ratio;

        let near_height = cam.near * fov_tan;
        let near_width = near_height * aspect_ratio;
        let far_height = cam.far * fov_tan;
        let far_width = far_height * aspect_ratio;

        // Compute the center points of the near and far planes
        let near_center = cam.position + cam.direction * cam.near;
        let far_center = cam.position + cam.direction * cam.far;

        // Define the 6 planes of the frustum using `FrustumPlane::from_points`
        Self {
            planes: [
                FrustumPlane::from_points(
                    near_center + cam.up * near_height - cam.right * near_width, // Left plane
                    near_center - cam.up * near_height - cam.right * near_width,
                    far_center + cam.up * far_height - cam.right * far_width,
                ),
                FrustumPlane::from_points(
                    near_center + cam.up * near_height + cam.right * near_width, // Right plane
                    near_center - cam.up * near_height + cam.right * near_width,
                    far_center + cam.up * far_height + cam.right * far_width,
                ),
                FrustumPlane::from_points(
                    near_center - cam.up * near_height - cam.right * near_width, // Bottom plane
                    near_center - cam.up * near_height + cam.right * near_width,
                    far_center - cam.up * far_height - cam.right * far_width,
                ),
                FrustumPlane::from_points(
                    near_center + cam.up * near_height - cam.right * near_width, // Top plane
                    near_center + cam.up * near_height + cam.right * near_width,
                    far_center + cam.up * far_height + cam.right * far_width,
                ),
                FrustumPlane::from_points(
                    near_center - cam.right * near_width + cam.up * near_height, // Near plane
                    near_center + cam.right * near_width + cam.up * near_height,
                    near_center + cam.right * near_width - cam.up * near_height,
                ),
                FrustumPlane::from_points(
                    far_center - cam.right * far_width + cam.up * far_height, // Far plane
                    far_center + cam.right * far_width + cam.up * far_height,
                    far_center + cam.right * far_width - cam.up * far_height,
                ),
            ],
            points: [Vec3::ZERO; POINT_COUNT], // Placeholder, define points if needed
        }
    }
}

