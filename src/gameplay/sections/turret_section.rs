//! A turret section is a component that can be added to an entity to give it a turret-like
//! behavior.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::transform::prelude::*;

pub mod prelude {
    pub use super::turret_section;
    pub use super::TurretSectionConfig;
    pub use super::TurretSectionMarker;
    pub use super::TurretSectionPlugin;
    pub use super::TurretSectionRotatorPitchMarker;
    pub use super::TurretSectionRotatorYawMarker;
    pub use super::TurretSectionTargetInput;
}

/// Configuration for a turret section of a spaceship.
#[derive(Clone, Debug)]
pub struct TurretSectionConfig {
    /// The transform of the turret section relative to its parent.
    pub transform: Transform,
    pub yaw_speed: f32,
    pub pitch_speed: f32,
    pub min_pitch: Option<f32>,
    pub max_pitch: Option<f32>,
}

impl Default for TurretSectionConfig {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            yaw_speed: std::f32::consts::PI, // 180 degrees per second
            pitch_speed: std::f32::consts::PI, // 180 degrees per second
            min_pitch: Some(-std::f32::consts::FRAC_PI_6),
            max_pitch: None,
        }
    }
}

/// Helper function to create a turret section entity bundle.
pub fn turret_section(config: TurretSectionConfig) -> impl Bundle {
    debug!("Creating turret section with config: {:?}", config);

    (
        Name::new("Turret Section"),
        TurretSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(1.0),
        TurretSectionTargetInput(None),
        config.transform,
        Visibility::Visible,
        children![(
            Name::new("Turret Section Rotator"),
            TurretSectionRotatorMarker,
            SmoothLookRotation {
                axis: Vec3::Y,
                initial: 0.0,
                speed: std::f32::consts::PI,
                ..default()
            },
            Transform::from_xyz(0.0, -0.5, 0.0),
            Visibility::Inherited,
            children![(
                Name::new("Turret Rotator Yaw"),
                TurretSectionRotatorYawMarker,
                SmoothLookRotation {
                    axis: Vec3::X,
                    initial: 0.0,
                    speed: std::f32::consts::PI,
                    min: config.min_pitch,
                    max: config.max_pitch,
                },
                Transform::from_xyz(0.0, 0.4, 0.0),
                Visibility::Inherited,
                children![(
                    Name::new("Turret Rotator Pitch"),
                    TurretSectionRotatorPitchMarker,
                    Transform::default(),
                    Visibility::Inherited,
                )],
            ),],
        )],
    )
}

/// Marker component for turret sections.
#[derive(Component, Clone, Debug, Reflect)]
pub struct TurretSectionMarker;

/// Marker component for the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretSectionRotatorMarker;

/// Marker component for the yaw part of the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretSectionRotatorYawMarker;

/// Marker component for the pitch part of the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretSectionRotatorPitchMarker;

/// The target input for the turret section. This is a world-space position that the turret will
/// aim at. If None, the turret will not rotate.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct TurretSectionTargetInput(pub Option<Vec3>);

/// A system set that will contain all the systems related to the turret section plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TurretSectionPluginSet;

/// A plugin that enables the TurretSection component and its related systems.
pub struct TurretSectionPlugin;

impl Plugin for TurretSectionPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: How can we check that the SmoothLookRotationPlugin is added?
        app.register_type::<TurretSectionMarker>()
            .register_type::<TurretSectionTargetInput>()
            .register_type::<TurretSectionRotatorYawMarker>()
            .register_type::<TurretSectionRotatorPitchMarker>();
        // TODO: Might add a flag for this later
        app.add_observer(insert_turret_section_render);
        app.add_observer(insert_turret_yaw_rotator_render);
        app.add_observer(insert_turret_pitch_rotator_render);

        app.add_systems(
            Update,
            // TODO: put the turret plugin between
            (
                update_turret_target_system,
                sync_turret_rotator_yaw_system,
                sync_turret_rotator_pitch_system,
            )
                .chain()
                .in_set(TurretSectionPluginSet),
        );
    }
}

fn update_turret_target_system(
    q_turret: Query<&TurretSectionTargetInput, With<TurretSectionMarker>>,
    mut q_yaw_rotator_base: Query<
        (&GlobalTransform, &mut SmoothLookRotationTarget, &ChildOf),
        With<TurretSectionRotatorMarker>,
    >,
    mut q_pitch_rotator_base: Query<
        (&GlobalTransform, &mut SmoothLookRotationTarget, &ChildOf),
        (
            With<TurretSectionRotatorYawMarker>,
            Without<TurretSectionRotatorMarker>,
        ),
    >,
    // TODO: Maybe we want to add a "barrel" marker that will be pointing to the target
    // to make it more realistic
) {
    for (pitch_base_transform, mut pitch_rotator_target, &ChildOf(entity)) in
        &mut q_pitch_rotator_base
    {
        let Ok((yaw_base_transform, mut yaw_rotator_target, &ChildOf(entity))) =
            q_yaw_rotator_base.get_mut(entity)
        else {
            warn!("TurretSectionRotatorPitch's parent does not have a TurretSectionRotatorMarker component");
            continue;
        };

        let Ok(target_input) = q_turret.get(entity) else {
            warn!("TurretSectionRotator's parent does not have a TurretSectionMarker component");
            continue;
        };

        let Some(target_input) = **target_input else {
            continue;
        };

        let world_to_yaw_base = yaw_base_transform.to_matrix().inverse();
        let world_to_pitch_base = pitch_base_transform.to_matrix().inverse();
        let world_pos = target_input;

        let yaw_local_pos = world_to_yaw_base.transform_point3(world_pos);
        let yaw_local_dir = yaw_local_pos.normalize_or_zero();

        let pitch_local_pos = world_to_pitch_base.transform_point3(world_pos);
        let pitch_local_dir = pitch_local_pos.normalize_or_zero();

        let (yaw, _, _) =
            Quat::from_rotation_arc(Vec3::NEG_Z, yaw_local_dir).to_euler(EulerRot::YXZ);
        let (_, pitch, _) =
            Quat::from_rotation_arc(Vec3::NEG_Z, pitch_local_dir).to_euler(EulerRot::YXZ);

        **yaw_rotator_target = yaw;
        **pitch_rotator_target = pitch;
    }
}

fn sync_turret_rotator_yaw_system(
    q_base: Query<&SmoothLookRotationOutput, With<TurretSectionRotatorMarker>>,
    mut q_yaw_rotator: Query<(&mut Transform, &ChildOf), With<TurretSectionRotatorYawMarker>>,
) {
    for (mut yaw_transform, &ChildOf(entity)) in &mut q_yaw_rotator {
        let Ok(rotator_output) = q_base.get(entity) else {
            warn!("TurretSectionRotatorYaw's parent does not have a TurretSectionRotatorMarker component");
            continue;
        };

        // Sync yaw rotator
        yaw_transform.rotation = Quat::from_euler(EulerRot::YXZ, **rotator_output, 0.0, 0.0);
    }
}

fn sync_turret_rotator_pitch_system(
    q_yaw: Query<&SmoothLookRotationOutput, With<TurretSectionRotatorYawMarker>>,
    mut q_pitch_rotator: Query<(&mut Transform, &ChildOf), With<TurretSectionRotatorPitchMarker>>,
) {
    for (mut pitch_transform, &ChildOf(entity)) in &mut q_pitch_rotator {
        let Ok(rotator_output) = q_yaw.get(entity) else {
            warn!("TurretSectionRotatorPitch's parent does not have a TurretSectionRotatorYawMarker component");
            continue;
        };

        // Sync pitch rotator
        pitch_transform.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, **rotator_output, 0.0);
    }
}

fn insert_turret_section_render(
    add: On<Add, TurretSectionRotatorMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = add.entity;
    debug!("Inserting render for TurretSection: {:?}", entity);

    commands.entity(entity).insert((
        Visibility::Inherited,
        children![(
            Name::new("Turret Base"),
            Transform::from_xyz(0.0, 0.15, 0.0),
            Mesh3d(meshes.add(Cylinder::new(0.5, 0.3))),
            MeshMaterial3d(materials.add(Color::srgb(0.25, 0.25, 0.25))),
        ),],
    ));
}

fn insert_turret_yaw_rotator_render(
    add: On<Add, TurretSectionRotatorYawMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = add.entity;
    debug!("Inserting render for TurretSection: {:?}", entity);

    commands.entity(entity).insert((
        Visibility::Inherited,
        children![(
            Name::new("Turret Yaw"),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Inherited,
            children![(
                Name::new("Turret Rotor"),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Mesh3d(meshes.add(Cylinder::new(0.4, 0.2))),
                MeshMaterial3d(materials.add(Color::srgb(0.4, 0.4, 0.4))),
            ),],
        ),],
    ));
}

fn insert_turret_pitch_rotator_render(
    add: On<Add, TurretSectionRotatorPitchMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = add.entity;
    debug!("Inserting render for TurretSection: {:?}", entity);

    commands.entity(entity).insert((
        Visibility::Inherited,
        children![(
            Name::new("Turret Pitch"),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::Inherited,
            children![
                // Pivot housing (sphere or dome)
                (
                    Name::new("Turret Pivot"),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    Mesh3d(meshes.add(Sphere::new(0.35))),
                    MeshMaterial3d(materials.add(Color::srgb(0.6, 0.6, 0.6))),
                ),
                // Gun body
                (
                    Name::new("Turret Body"),
                    Transform::from_xyz(0.0, 0.0, -0.35),
                    Mesh3d(meshes.add(Cuboid::new(0.28, 0.2, 0.5))), // shorter, chunkier body
                    MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.5))),
                    children![
                        // Shorter barrel
                        (
                            Name::new("Turret Barrel"),
                            Transform::from_xyz(0.0, 0.0, -0.5),
                            Mesh3d(meshes.add(Cuboid::new(0.12, 0.12, 0.7))),
                            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.7))),
                            children![
                                // Barrel tip
                                (
                                    Name::new("Barrel Tip"),
                                    Transform::from_xyz(0.0, 0.0, -0.4).with_rotation(
                                        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                                    ),
                                    Mesh3d(meshes.add(Cone::new(0.08, 0.18))),
                                    MeshMaterial3d(materials.add(Color::srgb(0.9, 0.2, 0.2))),
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    ));
}
