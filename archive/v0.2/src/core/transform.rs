use glam::{Mat4, Quat, Vec3};
use std::cell::RefCell;

/// A transform component that represents position, rotation, and scale in 3D space.
///
/// Uses interior mutability pattern with RefCells to allow for modification of the transform
/// while maintaining shared references. The model matrix is lazily computed and cached
/// when any of the transform components change.
#[derive(Debug, Clone)]
pub struct Transform {
    /// The position of the transform in world space
    pub pos: RefCell<Vec3>,

    /// The rotation of the transform as a quaternion
    pub rot: RefCell<Quat>,

    /// The scale of the transform on each axis
    pub scale: RefCell<Vec3>,

    /// The cached model matrix, computed when needed
    model_mat: RefCell<Mat4>,

    /// Flag indicating whether the model matrix needs to be recomputed
    model_mat_dirty: RefCell<bool>,
    pub dirty: RefCell<bool>,
}

impl Transform {
    /// Creates a new Transform with default values:
    /// - Position: (0, 0, 0)
    /// - Rotation: Identity quaternion (no rotation)
    /// - Scale: (1, 1, 1)
    ///
    /// The model matrix is initially marked as dirty and will be computed on first access.
    pub fn new() -> Self {
        let out = Transform {
            pos: RefCell::new(Vec3::ZERO),
            rot: RefCell::new(Quat::IDENTITY),
            scale: RefCell::new(Vec3::ONE),
            model_mat: RefCell::new(Mat4::ZERO),
            model_mat_dirty: RefCell::new(true),
            dirty: RefCell::new(false),
        };
        _ = out.model_mat();
        out
    }

    /// Gets the current model matrix, recomputing it if necessary.
    ///
    /// The model matrix transforms vertices from local space to world space,
    /// combining the effects of position, rotation, and scale.
    pub fn model_mat(&self) -> Mat4 {
        if *self.model_mat_dirty.borrow() {
            self.update_model_mat();
        }
        *self.model_mat.borrow()
    }

    /// Updates the cached model matrix using the current transform components.
    fn update_model_mat(&self) {
        *self.model_mat.borrow_mut() = Mat4::from_scale_rotation_translation(
            *self.scale.borrow(),
            *self.rot.borrow(),
            *self.pos.borrow(),
        );
        *self.model_mat_dirty.borrow_mut() = false;
    }

    /// Translates the transform by a given offset vector.
    ///
    /// # Arguments
    /// * `offset` - The vector to translate by in world space
    pub fn translate(&self, offset: Vec3) {
        *self.pos.borrow_mut() += offset;
        self.mark_dirty();
    }

    /// Scales the transform uniformly on all axes.
    ///
    /// # Arguments
    /// * `factor` - The uniform scale factor to apply
    pub fn scale_uniform(&self, factor: f32) {
        *self.scale.borrow_mut() *= factor;
        self.mark_dirty();
    }

    /// Scales the transform non-uniformly along each axis.
    ///
    /// # Arguments
    /// * `scale_factors` - Vector containing scale factors for each axis
    pub fn scale_non_uniform(&self, scale_factors: Vec3) {
        *self.scale.borrow_mut() *= scale_factors;
        self.mark_dirty();
    }

    /// Rotates the transform by applying a quaternion rotation.
    ///
    /// # Arguments
    /// * `rotation` - The quaternion representing the rotation to apply
    pub fn rotate_quat(&self, rotation: Quat) {
        let new_rot = {
            let current_rot = self.rot.borrow();
            rotation * *current_rot
        };
        *self.rot.borrow_mut() = new_rot;
        *self.model_mat_dirty.borrow_mut() = true;
        self.mark_dirty();
    }
    /// Rotates the transform using Euler angles.
    ///
    /// # Arguments
    /// * `pitch` - Rotation around the X axis (in radians)
    /// * `yaw` - Rotation around the Y axis (in radians)
    /// * `roll` - Rotation around the Z axis (in radians)
    pub fn rotate_euler(&self, pitch: f32, yaw: f32, roll: f32) {
        let rotation = Quat::from_euler(glam::EulerRot::XYZ, pitch, yaw, roll);
        self.rotate_quat(rotation);
    }

    /// Marks the model matrix as dirty, triggering a recomputation on next access.
    fn mark_dirty(&self) {
        self.model_mat_dirty.replace(true);
        self.dirty.replace(true);
    }
    pub fn mark_clean(&self) {
        self.dirty.replace(false);
        self.model_mat_dirty.replace(false);
    }

    pub fn is_dirty(&self) -> bool {
        *self.dirty.borrow()
    }
}

impl Default for Transform {
    /// Creates a new Transform with default values.
    fn default() -> Self {
        Self::new()
    }
}
