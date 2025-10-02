use bevy::prelude::*;

pub fn spaceship_render(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> impl Bundle {
    children![
        (
            Name::new("Spaceship Renderer"),
            Mesh3d(meshes.add(Cylinder::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ),
        (
            Name::new("Spaceship Thruster"),
            Mesh3d(meshes.add(Cone::new(0.5, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
            Transform::from_xyz(0.0, 0.0, 0.5)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ),
        (
            Name::new("Spaceship Window"),
            Mesh3d(meshes.add(Cylinder::new(0.2, 0.1))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 1.0))),
            Transform::from_xyz(0.0, 0.5, -0.1),
        )
    ]
}

pub fn turret_render(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> impl Bundle {
    children![
        // Base
        (
            Name::new("Turret Base"),
            Transform::default(),
            Mesh3d(meshes.add(Cylinder::new(0.6, 0.3))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
        ),
        // Yaw rotor / mount
        (
            Name::new("Turret Rotor"),
            Transform::from_xyz(0.0, 0.15, 0.0),
            Mesh3d(meshes.add(Cylinder::new(0.4, 0.1))),
            MeshMaterial3d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        ),
        // Sphere for pivot point
        (
            Name::new("Turret Pivot"),
            Transform::from_xyz(0.0, -0.2, 0.0),
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.7, 0.7, 0.7))),
        ),
        // Main Barrel
        (
            Name::new("Turret Barrel"),
            Transform::from_xyz(0.0, 0.0, -0.8),
            Mesh3d(meshes.add(Cuboid::new(0.15, 0.15, 1.2))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.7))),
            children![
                // Barrel tip
                (
                    Name::new("Barrel Tip"),
                    Transform::from_xyz(0.0, 0.0, -0.6),
                    Mesh3d(meshes.add(Cone::new(0.1, 0.2))),
                    MeshMaterial3d(materials.add(Color::srgb(0.9, 0.2, 0.2))),
                ),
                // Optional second barrel (for twin cannons)
                (
                    Name::new("Second Barrel"),
                    Transform::from_xyz(0.2, 0.0, -0.4),
                    Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.8))),
                    MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.7))),
                ),
            ],
        ),
        // Small detail lights on the base
        (
            Name::new("Base Lights"),
            Transform::from_xyz(0.35, 0.0, 0.0),
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
        ),
        (
            Name::new("Base Lights 2"),
            Transform::from_xyz(-0.35, 0.0, 0.0),
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
        ),
    ]
}
