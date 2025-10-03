//! A hull section is a modular part of a ship's hull. It just adds a physical body to which other
//! can connect. It represents the basic building block of a ship's structure.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::transform::prelude::*;

pub mod prelude {
    pub use super::turret_section;
    pub use super::TurretSectionConfig;
    pub use super::TurretSectionMarker;
    pub use super::TurretSectionPlugin;
    pub use super::TurretSectionRotatorMarker;
    pub use super::TurretSectionTargetInput;
}

#[derive(Default, Clone, Debug)]
pub struct TurretSectionConfig {
    pub transform: Transform,
}

pub fn turret_section(config: TurretSectionConfig) -> impl Bundle {
    (
        Name::new("Turret Section"),
        TurretSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(1.0),
        TurretSectionTargetInput(None),
        config.transform,
        Visibility::Visible,
        children![(
            Name::new("Turret Rotator"),
            TurretSectionRotatorMarker,
            SmoothLookRotation {
                initial_yaw: 0.0,
                initial_pitch: 0.0,
                yaw_speed: std::f32::consts::PI, // 180 degrees per second
                pitch_speed: std::f32::consts::PI, // 180 degrees per second
                min_pitch: Some(-std::f32::consts::FRAC_PI_6),
                max_pitch: None,
            },
            Transform::default(),
            Visibility::Inherited,
        ),],
    )
}

/// This will be root component for the turret entity. It will hold as a child the rotation part
/// and it will provide a reference frame for the rotation.
#[derive(Component, Clone, Debug, Reflect)]
pub struct TurretSectionMarker;

/// This will be the component for the rotating part of the turret. It will be a child of the base.
/// NOTE: I will want to separate this into yaw and pitch rotators later, but for now this is
/// sufficient.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretSectionRotatorMarker;

/// This will be the turret's target component input. It will be a Vec3 target position that we
/// want to aim at in world space.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct TurretSectionTargetInput(pub Option<Vec3>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TurretSectionPluginSet;

pub struct TurretSectionPlugin;

impl Plugin for TurretSectionPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: How can we check that the SmoothLookRotationPlugin is added?
        app.register_type::<TurretSectionMarker>()
            .register_type::<TurretSectionRotatorMarker>()
            .register_type::<TurretSectionTargetInput>();
        // TODO: Might add a flag for this later
        app.add_observer(insert_turret_section_render);

        app.add_systems(
            Update,
            // TODO: put the turret plugin between
            (update_turret_target_system, sync_turret_transform_system)
                .chain()
                .in_set(TurretSectionPluginSet),
        );
    }
}

fn update_turret_target_system(
    q_turret: Query<(&GlobalTransform, &TurretSectionTargetInput), With<TurretSectionMarker>>,
    mut q_rotator: Query<
        (&mut SmoothLookRotationTarget, &ChildOf),
        With<TurretSectionRotatorMarker>,
    >,
) {
    for (mut rotator_target, &ChildOf(parent)) in &mut q_rotator {
        let Ok((turret_transform, target_input)) = q_turret.get(parent) else {
            warn!("TurretRotatorMarker's parent is not a TurretSectionMarker");
            continue;
        };

        let Some(target_input) = **target_input else {
            continue;
        };

        let world_to_turret = turret_transform.compute_matrix().inverse();
        let world_pos = target_input;
        let local_pos = world_to_turret.transform_point3(world_pos);

        let dir_local = local_pos.normalize_or_zero();

        let (yaw, pitch, _) =
            Quat::from_rotation_arc(Vec3::NEG_Z, dir_local).to_euler(EulerRot::YXZ);

        rotator_target.yaw = yaw;
        rotator_target.pitch = pitch;
    }
}

fn sync_turret_transform_system(
    mut q_rotator: Query<
        (&SmoothLookRotationOutput, &mut Transform),
        With<TurretSectionRotatorMarker>,
    >,
) {
    for (output, mut transform) in &mut q_rotator {
        *transform = Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            output.yaw,
            output.pitch,
            0.0,
        ));
    }
}

fn insert_turret_section_render(
    trigger: Trigger<OnAdd, TurretSectionRotatorMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.target();
    debug!("Inserting render for TurretSection: {:?}", entity);

    commands.entity(entity).insert((
        Visibility::Visible,
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
        ],
    ));
}
