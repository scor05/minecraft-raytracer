use raylib::prelude::Vector3;

pub struct Camera {
    pub eye: Vector3,
    pub center: Vector3,
    pub up: Vector3,
    pub forward: Vector3,
    pub right: Vector3,
    changed: bool,
}

impl Camera {
    pub fn new(eye: Vector3, center: Vector3, up: Vector3) -> Self {
        let mut camera = Camera {
            eye,
            center,
            up,
            forward: Vector3::zero(),
            right: Vector3::zero(),
            changed: true,
        };
        camera.update_basis_vectors();
        camera
    }

    pub fn update_basis_vectors(&mut self) {
        self.forward = (self.center - self.eye).normalize();
        self.right = self.forward.cross(self.up).normalize();
        self.up = self.right.cross(self.forward);
        self.changed = true;
    }

    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let relative_pos = self.eye - self.center;
        let radius = relative_pos.length();
        let current_yaw = relative_pos.z.atan2(relative_pos.x);
        let current_pitch = (relative_pos.y / radius).asin();

        let new_yaw = current_yaw + yaw;
        let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5); // Clamp to avoid gimbal lock
        let cos_pitch = new_pitch.cos();
        let new_relative_pos = Vector3::new(
            radius * cos_pitch * new_yaw.cos(), // X component
            radius * new_pitch.sin(),           // Y component
            radius * cos_pitch * new_yaw.sin(), // Z component
        );
        self.eye = self.center + new_relative_pos;
        self.update_basis_vectors();
    }

    pub fn zoom(&mut self, amount: f32) {
        let to_center = self.center - self.eye;
        let distance = to_center.length();
        let forward = to_center.normalize();
        // Keep the eye from crossing the focal point when zooming in.
        let amount = amount.min((distance - 0.1).max(0.0));
        self.eye += forward * amount;
        self.update_basis_vectors();
    }

    pub fn move_forward(&mut self, amount: f32) {
        let offset = self.forward * amount;
        self.eye += offset;
        self.center += offset;
        self.update_basis_vectors();
    }

    pub fn move_up(&mut self, amount: f32) {
        let offset = self.up * amount;
        self.eye += offset;
        self.center += offset;
        self.update_basis_vectors();
    }

    pub fn move_right(&mut self, amount: f32) {
        let offset = self.right * amount;
        self.eye += offset;
        self.center += offset;
        self.update_basis_vectors();
    }

    pub fn is_changed(&mut self) -> bool {
        let changed = self.changed;
        self.changed = false;
        changed
    }

    pub fn basis_change(&self, v: &Vector3) -> Vector3 {
        Vector3::new(
            v.x * self.right.x + v.y * self.up.x - v.z * self.forward.x,
            v.x * self.right.y + v.y * self.up.y - v.z * self.forward.y,
            v.x * self.right.z + v.y * self.up.z - v.z * self.forward.z,
        )
    }
}
