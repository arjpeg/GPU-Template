use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};
use winit::{dpi::PhysicalSize, keyboard::KeyCode};

/// A first person camera without roll.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    /// The current position, also called the eye of the camera.
    pub position: Vec3,

    /// The rotation around y axis (in radians).
    pub yaw: f32,
    /// The rotation around x axis (in radians).
    pub pitch: f32,

    /// The vertical field of view, or what extent of the world can be seen (in radians).
    pub fov: f32,
    /// The current aspect ratio of the rendering surface.
    pub aspect_ratio: f32,

    /// How fast the player can move the control across space.
    pub movement_sensitivity: f32,
    /// How fast the camera responds to rotated by the mouse.
    pub mouse_sensitivity: f32,
}

impl Camera {
    /// Returns the current view-projection transformation matrix.
    pub fn view_projection(&self) -> Mat4 {
        let projection = Mat4::perspective_infinite_rh(self.fov, self.aspect_ratio, 0.1);
        let view = Mat4::look_to_rh(self.position, self.forward(), Vec3::Y);

        projection * view
    }

    /// Returns the forward vector, or the current direction of the camera.
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }

    /// Updates the camera's position based on the user's input.
    pub fn update_position(&mut self, key_down: impl Fn(&KeyCode) -> bool, dt: f32) {
        let up = Vec3::Y;

        let forward_xz = self.forward().with_y(0.0);
        let right_xz = forward_xz.cross(up);

        let direction_mapping = [
            (KeyCode::KeyW, forward_xz),
            (KeyCode::KeyS, -forward_xz),
            (KeyCode::KeyD, right_xz),
            (KeyCode::KeyA, -right_xz),
            (KeyCode::Space, up),
            (KeyCode::ShiftLeft, -up),
        ];

        let delta_position = direction_mapping
            .iter()
            .filter_map(|(code, direction)| key_down(code).then_some(direction))
            .sum::<Vec3>()
            .normalize_or_zero();

        let speed_boost = if key_down(&KeyCode::ControlLeft) {
            1.5
        } else {
            1.0
        };

        self.position += delta_position * dt * self.movement_sensitivity * speed_boost;
    }

    /// Updates the camera's orientation (yaw and pitch) based on the user's input.
    pub fn update_orientation(&mut self, delta: (f32, f32)) {
        let (dx, dy) = delta;

        self.pitch -= dy * self.mouse_sensitivity;
        self.yaw += dx * self.mouse_sensitivity;

        self.pitch = self.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    }

    /// Resizes the camera's aspect ratio to match the new window size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = size;

        self.aspect_ratio = width as f32 / height as f32;
    }
}
