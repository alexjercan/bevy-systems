use bevy::prelude::*;

/// Trait for interpolating a value smoothly towards a target with optional snapping.
///
/// The `lerp_and_snap` method allows for smooth interpolation over time while ensuring
/// that the value snaps exactly to the target when it is very close. This is useful for
/// camera movement, smoothing physics values, or any gradual transitions.
pub trait LerpSnap {
    /// Interpolates `self` towards `to` using the given `smoothness` and `dt` (delta time),
    /// and snaps to `to` if close enough.
    ///
    /// - `to`: The target value to interpolate towards.
    /// - `smoothness`: A value between 0.0 and 1.0 controlling how smooth the interpolation is.
    ///   0.0 means instant change, 1.0 means very smooth.
    /// - `dt`: The delta time since the last update.
    ///
    /// Returns the new interpolated (and possibly snapped) value.
    fn lerp_and_snap(&self, to: Self, smoothness: f32, dt: f32) -> Self;
}

impl LerpSnap for f32 {
    fn lerp_and_snap(&self, to: Self, smoothness: f32, dt: f32) -> Self {
        // Adjust smoothing factor for exponential effect
        let t = smoothness.powi(7);
        // Interpolate using Bevy's built-in lerp
        let mut new_value = self.lerp(to, 1.0 - t.powf(dt));
        // Snap to target if very close and smoothing is not 1
        if smoothness < 1.0 && (new_value - to).abs() < f32::EPSILON {
            new_value = to;
        }

        new_value
    }
}

impl LerpSnap for Vec3 {
    fn lerp_and_snap(&self, to: Self, smoothness: f32, dt: f32) -> Self {
        // Adjust smoothing factor for exponential effect
        let t = smoothness.powi(7);
        // Interpolate using Bevy's built-in Vec3::lerp
        let mut new_value = self.lerp(to, 1.0 - t.powf(dt));
        // Snap to target if very close and smoothing is not 1
        if smoothness < 1.0 && (new_value - to).length() < f32::EPSILON {
            new_value = to;
        }

        new_value
    }
}
