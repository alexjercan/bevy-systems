use bevy::prelude::*;

/// Convert spherical coordinates to a 3D Cartesian vector.
///
/// - `radius`: Distance from the origin.
/// - `theta`: Horizontal angle in radians, measured from -Z axis around Y axis.
/// - `phi`: Vertical angle in radians from the horizontal plane.
///
/// Returns a `Vec3` representing the Cartesian coordinates.
pub fn spherical_to_cartesian(radius: f32, theta: f32, phi: f32) -> Vec3 {
    let x = radius * phi.cos() * theta.sin();
    let y = radius * phi.sin();
    let z = -radius * phi.cos() * theta.cos();
    Vec3::new(x, y, z)
}

/// Convert a direction vector to spherical coordinates (theta, phi).
///
/// - `direction`: The 3D direction vector to convert.
///
/// Returns `(theta, phi)` where:
/// - `theta` is the horizontal angle in radians around the Y axis from -Z,
/// - `phi` is the vertical angle in radians from the horizontal plane.
pub fn direction_to_spherical(direction: Vec3) -> (f32, f32) {
    let r = direction.length();
    if r == 0.0 {
        return (0.0, 0.0);
    }

    let x = direction.x / r;
    let y = direction.y / r;
    let z = direction.z / r;

    let horiz = (x * x + z * z).sqrt();
    let eps = 1e-6_f32;

    let theta = if horiz <= eps { 0.0 } else { x.atan2(-z) };
    let phi = y.atan2(horiz);

    (theta, phi)
}

/// Spherically interpolate between two vectors along the great circle.
///
/// - `a`: Start vector.
/// - `b`: End vector.
/// - `t`: Interpolation factor (0.0 = `a`, 1.0 = `b`).
///
/// Returns a unit vector on the great circle path from `a` to `b`.
pub fn slerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let dot = a.dot(b).clamp(-1.0, 1.0);
    let theta = dot.acos() * t;
    let relative = (b - a * dot).normalize();
    (a * theta.cos() + relative * theta.sin()).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_to_cartesian() {
        let radius = 1.0;

        // -Z
        let theta = 0.0;
        let phi = 0.0;
        let pos = spherical_to_cartesian(radius, theta, phi);
        assert!(pos.abs_diff_eq(Vec3::new(0.0, 0.0, -1.0), 1e-6));

        // +X
        let theta = std::f32::consts::FRAC_PI_2;
        let phi = 0.0;
        let pos = spherical_to_cartesian(radius, theta, phi);
        assert!(pos.abs_diff_eq(Vec3::new(1.0, 0.0, 0.0), 1e-6));

        // +Y
        let theta = 0.0;
        let phi = std::f32::consts::FRAC_PI_2;
        let pos = spherical_to_cartesian(radius, theta, phi);
        assert!(pos.abs_diff_eq(Vec3::new(0.0, 1.0, 0.0), 1e-6));
    }

    #[test]
    fn test_direction_to_spherical() {
        // -Z
        let dir = Vec3::new(0.0, 0.0, -1.0);
        let (theta, phi) = direction_to_spherical(dir);
        assert!(theta.abs() <= 1e-6);
        assert!(phi.abs() <= 1e-6);

        // +X
        let dir = Vec3::new(1.0, 0.0, 0.0);
        let (theta, phi) = direction_to_spherical(dir);
        assert!((theta - std::f32::consts::FRAC_PI_2).abs() <= 1e-6);
        assert!(phi.abs() <= 1e-6);

        // +Y
        let dir = Vec3::new(0.0, 1.0, 0.0);
        let (theta, phi) = direction_to_spherical(dir);
        assert!(theta.abs() <= 1e-6);
        assert!((phi - std::f32::consts::FRAC_PI_2).abs() <= 1e-6);
    }
}
