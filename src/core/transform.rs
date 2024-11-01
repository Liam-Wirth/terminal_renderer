use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
    model_mat: Mat4,
    model_mat_dirty: bool,
}

impl Transform {
    pub fn new() -> Self {
        let mut out = Transform {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
            model_mat: Mat4::ZERO,
            model_mat_dirty: true,
        };
        _ = out.model_mat();
        out
    }
    pub fn model_mat(&mut self) -> Mat4 {
        if self.model_mat_dirty {
            self.update_model_mat();
        }
        self.model_mat
    }

    fn update_model_mat(&mut self) {
        self.model_mat = Mat4::from_scale_rotation_translation(self.scale, self.rot, self.pos);
        self.model_mat_dirty = false;
    }
    /// Translate the model by a given offset
    pub fn translate(&mut self, offset: Vec3) {
        self.pos += offset;
        self.mark_dirty();
    }

    /// Uniformly scale the model by a given factor
    pub fn scale_uniform(&mut self, factor: f32) {
        self.scale *= factor;
        self.mark_dirty();
    }

    /// Scale the model non-uniformly by different factors for each axis
    pub fn scale_non_uniform(&mut self, scale_factors: Vec3) {
        self.scale *= scale_factors;
        self.mark_dirty();
    }

    /// Rotate the model by a quaternion
    pub fn rotate_quat(&mut self, rotation: Quat) {
        self.rot = rotation * self.rot;
        self.mark_dirty();
    }

    /// Rotate the model using Euler angles (in radians)
    pub fn rotate_euler(&mut self, pitch: f32, yaw: f32, roll: f32) {
        let rotation = Quat::from_euler(glam::EulerRot::XYZ, pitch, yaw, roll);
        self.rotate_quat(rotation);
    }

    /// Helper function to mark the model matrix as dirty (needs recalculating)
    fn mark_dirty(&mut self) {
        self.model_mat_dirty = true;
    }
}
