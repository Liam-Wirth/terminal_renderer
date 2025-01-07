use glam::{Vec3, Quat, Affine3A};

pub struct Ship {
    pub position: Vec3,
    pub velocity: Vec3,
    pub orientation: Quat,
    pub angular_velocity: Vec3,
    pub thrust: f32,
    pub roll_speed: f32,
    pub pitch_speed: f32,
    pub yaw_speed: f32,
    pub max_speed: f32,
    pub drag: f32,
}

impl Ship {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            angular_velocity: Vec3::ZERO,
            thrust: 20.0,
            roll_speed: 2.0,
            pitch_speed: 1.5,
            yaw_speed: 1.0,
            max_speed: 50.0,
            drag: 0.1,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Apply thrust in the forward direction
        let forward = self.orientation * Vec3::Z;
        self.velocity += forward * self.thrust * delta_time;

        // Apply drag
        self.velocity *= 1.0 - self.drag * delta_time;

        // Clamp velocity to max speed
        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }

        // Update position
        self.position += self.velocity * delta_time;

        // Apply angular velocity
        let rotation = Quat::from_rotation_x(self.angular_velocity.x * delta_time)
            * Quat::from_rotation_y(self.angular_velocity.y * delta_time)
            * Quat::from_rotation_z(self.angular_velocity.z * delta_time);
        self.orientation = (self.orientation * rotation).normalize();
    }

    pub fn get_transform(&self) -> Affine3A {
        Affine3A::from_rotation_translation(self.orientation, self.position)
    }
}

