//! TODO: Document this module and refactor

use bevy::prelude::*;

pub mod prelude {
    pub use super::LerpSnap;
}

pub trait LerpSnap {
    fn lerp_and_snap(&self, to: Self, smoothness: f32, dt: f32) -> Self;
}

impl LerpSnap for f32 {
    fn lerp_and_snap(&self, to: Self, smoothness: f32, dt: f32) -> Self {
        let t = smoothness.powi(7);
        let mut new_value = self.lerp(to, 1.0 - t.powf(dt));
        if smoothness < 1.0 && (new_value - to).abs() < f32::EPSILON {
            new_value = to;
        }

        new_value
    }
}

impl LerpSnap for Vec3 {
    fn lerp_and_snap(&self, to: Self, smoothness: f32, dt: f32) -> Self {
        let t = smoothness.powi(7);
        let mut new_value = self.lerp(to, 1.0 - t.powf(dt));
        if smoothness < 1.0 && (new_value - to).length() < f32::EPSILON {
            new_value = to;
        }

        new_value
    }
}
