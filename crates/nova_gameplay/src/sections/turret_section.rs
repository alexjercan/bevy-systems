//! A turret section is a component that can be added to an entity to give it a turret-like
//! behavior.

// TODO: Cleanup the magic numbers into constants

use avian3d::prelude::*;
use bevy::prelude::*;

use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::turret_section;
    pub use super::TurretSectionBarrelMuzzleEntity;
    pub use super::TurretSectionBarrelMuzzleMarker;
    pub use super::TurretSectionConfig;
    pub use super::TurretSectionMarker;
    pub use super::TurretSectionPlugin;
    pub use super::TurretSectionTargetInput;
}

const TURRET_SECTION_DEFAULT_COLLIDER_DENSITY: f32 = 1.0;

/// Configuration for a turret section of a spaceship.
#[derive(Clone, Debug, Reflect)]
pub struct TurretSectionConfig {
    /// The transform of the turret section relative to its parent.
    pub transform: Transform,
    /// The yaw speed of the turret section in radians per second.
    pub yaw_speed: f32,
    /// The pitch speed of the turret section in radians per second.
    pub pitch_speed: f32,
    /// The minimum pitch angle of the turret section in radians. If None, there is no limit.
    pub min_pitch: Option<f32>,
    /// The maximum pitch angle of the turret section in radians. If None, there is no limit.
    pub max_pitch: Option<f32>,
    /// The collider density / mass of the section.
    pub collider_density: f32,
    /// The render mesh of the base, defaults to a cylinder base
    pub render_mesh_base: Option<Handle<Scene>>,
    /// The offset of the base from the section origin
    pub base_offset: Vec3,
    /// The render mesh of the yaw rotator, defaults to a cylinder with ridges
    pub render_mesh_yaw: Option<Handle<Scene>>,
    /// The offset of the yaw rotator from the base
    pub yaw_offset: Vec3,
    /// The render mesh of the pitch rotator, defaults to a cylinder with ridges
    pub render_mesh_pitch: Option<Handle<Scene>>,
    /// The offset of the pitch rotator from the yaw rotator
    pub pitch_offset: Vec3,
    /// The render mesh of the barrel, defaults to a simple barrel shape
    pub render_mesh_barrel: Option<Handle<Scene>>,
    /// The offset of the barrel from the pitch rotator
    pub barrel_offset: Vec3,
    /// The offset of the muzzle from the barrel
    pub muzzle_offset: Vec3,
    /// The fire rate of the turret in rounds per second.
    pub fire_rate: f32,
    /// The projectile configuration for the bullets fired by the turret.
    pub projectile: BulletProjectileConfig,
}

impl Default for TurretSectionConfig {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            yaw_speed: std::f32::consts::PI, // 180 degrees per second
            pitch_speed: std::f32::consts::PI, // 180 degrees per second
            min_pitch: Some(-std::f32::consts::FRAC_PI_6),
            max_pitch: Some(std::f32::consts::FRAC_PI_2),
            collider_density: TURRET_SECTION_DEFAULT_COLLIDER_DENSITY,
            render_mesh_base: None,
            base_offset: Vec3::new(0.0, -0.5, 0.0),
            render_mesh_yaw: None,
            yaw_offset: Vec3::new(0.0, 0.1, 0.0),
            render_mesh_pitch: None,
            pitch_offset: Vec3::new(0.0, 0.2, 0.0),
            render_mesh_barrel: None,
            barrel_offset: Vec3::new(0.1, 0.2, 0.0),
            muzzle_offset: Vec3::new(0.0, 0.0, -0.5),
            fire_rate: 100.0,
            projectile: BulletProjectileConfig {
                muzzle_speed: 100.0,
                lifetime: 5.0,
                render_mesh: None,
            },
        }
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TurretSectionBaseRenderMesh(Option<Handle<Scene>>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TurretSectionYawRenderMesh(Option<Handle<Scene>>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TurretSectionPitchRenderMesh(Option<Handle<Scene>>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TurretSectionBarrelRenderMesh(Option<Handle<Scene>>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TurretSectionConfigHelper(TurretSectionConfig);

/// Helper function to create a turret section entity bundle.
pub fn turret_section(config: TurretSectionConfig) -> impl Bundle {
    debug!("Creating turret section with config: {:?}", config);

    (
        Name::new("Turret Section"),
        super::SectionMarker,
        TurretSectionMarker,
        Collider::cuboid(1.0, 1.0, 1.0),
        ColliderDensity(config.collider_density),
        TurretSectionTargetInput(None),
        config.transform,
        Visibility::Visible,
        TurretSectionConfigHelper(config),
    )
}

/// Marker component for turret sections.
#[derive(Component, Clone, Debug, Reflect)]
pub struct TurretSectionMarker;

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct TurretRotatorBaseMarker;

/// Marker component for the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
struct TurretSectionRotatorYawBaseMarker;

/// Marker component for the yaw part of the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
struct TurretSectionRotatorYawMarker;

/// Marker component for the pitch part of the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
struct TurretSectionRotatorPitchBaseMarker;

/// Marker component for the pitch part of the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
struct TurretSectionRotatorPitchMarker;

/// Marker component for the barrel part of the turret section rotator.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretSectionRotatorBarrelMarker;

/// Marker component for the muzzle of the barrel.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct TurretSectionBarrelMuzzleMarker;

/// The entity that represents the muzzle of the turret barrel.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct TurretSectionBarrelMuzzleEntity(pub Entity);

/// The target input for the turret section. This is a world-space position that the turret will
/// aim at. If None, the turret will not rotate.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct TurretSectionTargetInput(pub Option<Vec3>);

/// A system set that will contain all the systems related to the turret section plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TurretSectionPluginSet;

/// A plugin that enables the TurretSection component and its related systems.
#[derive(Default)]
pub struct TurretSectionPlugin {
    pub render: bool,
}

impl Plugin for TurretSectionPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(feature = "debug") {
            app.add_plugins(debug::DebugTurretSectionPlugin);
        }

        app.add_observer(insert_turret_section);

        // NOTE: How can we check that the SmoothLookRotationPlugin is added?
        if self.render {
            app.add_observer(insert_turret_section_render);
            app.add_observer(insert_turret_yaw_rotator_render);
            app.add_observer(insert_turret_pitch_rotator_render);
            app.add_observer(insert_turret_barrel_render);
        }

        app.add_systems(
            Update,
            // TODO: put the turret plugin between
            // NOTE: I have no idea what I meant here
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
    q_base: Query<&ChildOf, With<TurretRotatorBaseMarker>>,
    mut q_rotator_yaw_base: Query<
        (&GlobalTransform, &mut SmoothLookRotationTarget, &ChildOf),
        (
            With<TurretSectionRotatorYawBaseMarker>,
            Without<TurretSectionRotatorPitchBaseMarker>,
        ),
    >,
    q_rotator_yaw: Query<&ChildOf, With<TurretSectionRotatorYawMarker>>,
    mut q_rotator_pitch_base: Query<
        (&GlobalTransform, &mut SmoothLookRotationTarget, &ChildOf),
        (
            With<TurretSectionRotatorPitchBaseMarker>,
            Without<TurretSectionRotatorYawBaseMarker>,
        ),
    >,
    q_rotator_pitch: Query<&ChildOf, With<TurretSectionRotatorPitchMarker>>,
    q_barrel: Query<(&GlobalTransform, &ChildOf), With<TurretSectionRotatorBarrelMarker>>,
) {
    for (barrel_transform, &ChildOf(entity)) in &q_barrel {
        let Ok(&ChildOf(entity)) = q_rotator_pitch.get(entity) else {
            warn!("TurretSectionRotatorBarrel's parent does not have a TurretSectionRotatorPitchMarker component");
            continue;
        };

        let Ok((pitch_base_transform, mut pitch_rotator_target, &ChildOf(entity))) =
            q_rotator_pitch_base.get_mut(entity)
        else {
            warn!("TurretSectionRotatorPitch's parent does not have a TurretSectionRotatorPitchBaseMarker component");
            continue;
        };

        let Ok(&ChildOf(entity)) = q_rotator_yaw.get(entity) else {
            warn!("TurretSectionRotatorPitchBase's parent does not have a TurretSectionRotatorYawMarker component");
            continue;
        };

        let Ok((yaw_base_transform, mut yaw_rotator_target, &ChildOf(entity))) =
            q_rotator_yaw_base.get_mut(entity)
        else {
            warn!("TurretSectionRotatorYaw's parent does not have a TurretSectionRotatorYawBaseMarker component");
            continue;
        };

        let Ok(&ChildOf(entity)) = q_base.get(entity) else {
            warn!("TurretSectionRotatorYawBase's parent does not have a TurretRotatorBaseMarker component");
            continue;
        };

        let Ok(target_input) = q_turret.get(entity) else {
            warn!("TurretRotatorBase's parent does not have a TurretSectionMarker component");
            continue;
        };

        let Some(target_input) = **target_input else {
            continue;
        };

        let world_to_yaw_base = yaw_base_transform.to_matrix().inverse();
        let world_to_pitch_base = pitch_base_transform.to_matrix().inverse();

        let target_pos = target_input;
        let barrel_pos = barrel_transform.translation();
        let barrel_dir = barrel_transform.forward().into();
        if target_pos == barrel_pos {
            continue;
        }

        let barrel_yaw_local_pos = world_to_yaw_base.transform_point3(barrel_pos);
        let target_yaw_local_pos = world_to_yaw_base.transform_point3(target_pos);
        let barrel_yaw_local_dir = world_to_yaw_base.transform_vector3(barrel_dir);

        // phi is the angle from the x axis to the (x,-z) position
        let phi = (-target_yaw_local_pos.z).atan2(target_yaw_local_pos.x);
        // r is the distance from the origin to the barrel direction projected onto the xz plane
        let r = barrel_yaw_local_pos.cross(barrel_yaw_local_dir).y;
        let target_r = target_yaw_local_pos.xz().length();
        if target_r > r.abs() {
            let theta = (phi - (r / target_r).acos()) % (std::f32::consts::TAU);
            **yaw_rotator_target = theta;
        }

        let barrel_pitch_local_pos =
            world_to_pitch_base.transform_point3(barrel_transform.translation());
        let target_pitch_local_pos = world_to_pitch_base.transform_point3(target_input);
        let barrel_pitch_local_dir =
            world_to_pitch_base.transform_vector3(barrel_transform.forward().into());

        let phi = (-target_pitch_local_pos.z).atan2(target_pitch_local_pos.y);
        let r = -barrel_pitch_local_pos.cross(barrel_pitch_local_dir).x;
        let target_r = target_pitch_local_pos.yz().length();
        if target_r > r.abs() {
            let theta = phi - (r / target_r).acos();
            **pitch_rotator_target = -theta;
        }
    }
}

fn sync_turret_rotator_yaw_system(
    q_base: Query<&SmoothLookRotationOutput, With<TurretSectionRotatorYawBaseMarker>>,
    mut q_yaw_rotator: Query<(&mut Transform, &ChildOf), With<TurretSectionRotatorYawMarker>>,
) {
    for (mut yaw_transform, &ChildOf(entity)) in &mut q_yaw_rotator {
        let Ok(rotator_output) = q_base.get(entity) else {
            warn!("TurretSectionRotatorYaw's parent does not have a TurretSectionRotatorMarker component");
            continue;
        };

        yaw_transform.rotation = Quat::from_euler(EulerRot::YXZ, **rotator_output, 0.0, 0.0);
    }
}

fn sync_turret_rotator_pitch_system(
    q_base: Query<&SmoothLookRotationOutput, With<TurretSectionRotatorPitchBaseMarker>>,
    mut q_pitch_rotator: Query<(&mut Transform, &ChildOf), With<TurretSectionRotatorPitchMarker>>,
) {
    for (mut pitch_transform, &ChildOf(entity)) in &mut q_pitch_rotator {
        let Ok(rotator_output) = q_base.get(entity) else {
            warn!("TurretSectionRotatorPitch's parent does not have a TurretSectionRotatorPitchBaseMarker component");
            continue;
        };

        pitch_transform.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, **rotator_output, 0.0);
    }
}

fn insert_turret_section(
    add: On<Add, TurretSectionMarker>,
    mut commands: Commands,
    q_config: Query<&TurretSectionConfigHelper, With<TurretSectionMarker>>,
) {
    let entity = add.entity;
    debug!("Inserting turret section for entity: {:?}", entity);

    let Ok(config) = q_config.get(entity) else {
        warn!(
            "TurretSection entity {:?} missing TurretSectionConfigHelper component",
            entity
        );
        return;
    };
    let config = (**config).clone();

    let muzzle = commands
        .spawn((
            Name::new("Turret Barrel Muzzle"),
            TurretSectionBarrelMuzzleMarker,
            Transform::from_translation(config.muzzle_offset),
            Visibility::Inherited,
            children![(
                projectile_spawner(ProjectileSpawnerConfig {
                    fire_rate: config.fire_rate,
                    projectile: config.projectile,
                    ..default()
                }),
            )]
        ))
        .id();

    commands
        .entity(entity)
        .insert((TurretSectionBarrelMuzzleEntity(muzzle),))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Turret Rotator Base"),
                    TurretRotatorBaseMarker,
                    Transform::from_translation(config.base_offset),
                    Visibility::Inherited,
                    TurretSectionBaseRenderMesh(config.render_mesh_base),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Name::new("Turret Rotator Yaw Base"),
                            TurretSectionRotatorYawBaseMarker,
                            SmoothLookRotation {
                                axis: Vec3::Y,
                                initial: 0.0,
                                speed: config.yaw_speed,
                                ..default()
                            },
                            Transform::from_translation(config.yaw_offset),
                            Visibility::Inherited,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    Name::new("Turret Rotator Yaw"),
                                    TurretSectionRotatorYawMarker,
                                    Transform::default(),
                                    Visibility::Inherited,
                                    TurretSectionYawRenderMesh(config.render_mesh_yaw),
                                ))
                                .with_children(|parent| {
                                    parent
                                        .spawn((
                                            Name::new("Turret Rotator Pitch Base"),
                                            TurretSectionRotatorPitchBaseMarker,
                                            SmoothLookRotation {
                                                axis: Vec3::X,
                                                initial: 0.0,
                                                speed: config.pitch_speed,
                                                min: config.min_pitch,
                                                max: config.max_pitch,
                                            },
                                            Transform::from_translation(config.pitch_offset),
                                            Visibility::Inherited,
                                        ))
                                        .with_children(|parent| {
                                            parent
                                                .spawn((
                                                    Name::new("Turret Rotator Pitch"),
                                                    TurretSectionRotatorPitchMarker,
                                                    Transform::default(),
                                                    Visibility::Inherited,
                                                    TurretSectionPitchRenderMesh(
                                                        config.render_mesh_pitch,
                                                    ),
                                                ))
                                                .with_children(|parent| {
                                                    parent
                                                        .spawn((
                                                            Name::new("Turret Rotator Barrel"),
                                                            TurretSectionRotatorBarrelMarker,
                                                            Transform::from_translation(
                                                                config.barrel_offset,
                                                            ),
                                                            Visibility::Inherited,
                                                            TurretSectionBarrelRenderMesh(
                                                                config.render_mesh_barrel,
                                                            ),
                                                        ))
                                                        .add_child(muzzle);
                                                });
                                        });
                                });
                        });
                });
        });
}

fn insert_turret_section_render(
    add: On<Add, TurretRotatorBaseMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_base: Query<&TurretSectionBaseRenderMesh, With<TurretRotatorBaseMarker>>,
) {
    let entity = add.entity;
    debug!("Inserting render for TurretRotatorBaseMarker: {:?}", entity);

    let Ok(render_mesh) = q_base.get(entity) else {
        warn!(
            "TurretRotatorBaseMarker entity {:?} missing TurretSectionBaseRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene) => {
            commands.entity(entity).insert((children![(
                Name::new("Render Turret Base"),
                SceneRoot(scene.clone()),
            ),],));
        }
        None => {
            commands.entity(entity).insert((children![(
                Name::new("Render Turret Base"),
                Transform::from_xyz(0.0, 0.05, 0.0),
                Mesh3d(meshes.add(Cylinder::new(0.5, 0.1))),
                MeshMaterial3d(materials.add(Color::srgb(0.25, 0.25, 0.25))),
            ),],));
        }
    }
}

fn insert_turret_yaw_rotator_render(
    add: On<Add, TurretSectionRotatorYawMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_yaw: Query<&TurretSectionYawRenderMesh, With<TurretSectionRotatorYawMarker>>,
) {
    let entity = add.entity;
    debug!(
        "Inserting render for TurretSectionRotatorYawMarker: {:?}",
        entity
    );

    let Ok(render_mesh) = q_yaw.get(entity) else {
        warn!(
            "TurretSectionRotatorYawMarker entity {:?} missing TurretSectionYawRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene) => {
            commands.entity(entity).insert((children![(
                Name::new("Render Turret Yaw"),
                SceneRoot(scene.clone()),
            ),],));
        }
        None => {
            let base_mat = materials.add(Color::srgb(0.4, 0.4, 0.4));
            let ridge_mat = materials.add(Color::srgb(0.3, 0.3, 0.3));

            let base_cylinder = meshes.add(Cylinder::new(0.2, 0.2));

            let ridge_count = 16;
            let ridge_radius = 0.22;
            let ridge_height = 0.2;
            let ridge_width = 0.04;
            let ridge_depth = 0.02;

            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn((
                        Name::new("Render Turret Yaw"),
                        Transform::from_xyz(0.0, 0.1, 0.0),
                        Visibility::Inherited,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Name::new("Yaw Base"),
                            Mesh3d(base_cylinder.clone()),
                            MeshMaterial3d(base_mat.clone()),
                        ));

                        for i in 0..ridge_count {
                            let angle = i as f32 / ridge_count as f32 * std::f32::consts::TAU;
                            parent.spawn((
                                Name::new(format!("Ridge {i}")),
                                Transform::from_xyz(
                                    angle.cos() * ridge_radius,
                                    0.0,
                                    angle.sin() * ridge_radius,
                                )
                                .with_rotation(Quat::from_rotation_y(angle)),
                                Mesh3d(meshes.add(Cuboid::new(
                                    ridge_depth,
                                    ridge_height,
                                    ridge_width,
                                ))),
                                MeshMaterial3d(ridge_mat.clone()),
                            ));
                        }
                    });
            });
        }
    }
}

fn insert_turret_pitch_rotator_render(
    add: On<Add, TurretSectionRotatorPitchMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_pitch: Query<&TurretSectionPitchRenderMesh, With<TurretSectionRotatorPitchMarker>>,
) {
    let entity = add.entity;
    debug!(
        "Inserting render for TurretSectionRotatorPitchMarker: {:?}",
        entity
    );

    let Ok(render_mesh) = q_pitch.get(entity) else {
        warn!(
            "TurretSectionRotatorPitchMarker entity {:?} missing TurretSectionPitchRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene) => {
            commands.entity(entity).insert((children![(
                Name::new("Render Turret Pitch"),
                SceneRoot(scene.clone()),
            ),],));
        }
        None => {
            let base_mat = materials.add(Color::srgb(0.5, 0.5, 0.5));
            let ridge_mat = materials.add(Color::srgb(0.3, 0.3, 0.3));

            let base_cylinder = meshes.add(Cylinder::new(0.2, 0.2));

            let ridge_count = 16;
            let ridge_radius = 0.22;
            let ridge_height = 0.2;
            let ridge_width = 0.04;
            let ridge_depth = 0.02;

            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn((
                        Name::new("Render Turret Pitch"),
                        Transform::from_xyz(0.3, 0.2, 0.0)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                        Visibility::Inherited,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Name::new("Pitch Base"),
                            Mesh3d(base_cylinder.clone()),
                            MeshMaterial3d(base_mat.clone()),
                        ));

                        for i in 0..ridge_count {
                            let angle = i as f32 / ridge_count as f32 * std::f32::consts::TAU;
                            parent.spawn((
                                Name::new(format!("Ridge {i}")),
                                Transform::from_xyz(
                                    angle.cos() * ridge_radius,
                                    0.0,
                                    angle.sin() * ridge_radius,
                                )
                                .with_rotation(Quat::from_rotation_y(angle)),
                                Mesh3d(meshes.add(Cuboid::new(
                                    ridge_depth,
                                    ridge_height,
                                    ridge_width,
                                ))),
                                MeshMaterial3d(ridge_mat.clone()),
                            ));
                        }
                    });
            });
        }
    }
}

fn insert_turret_barrel_render(
    add: On<Add, TurretSectionRotatorBarrelMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_barrel: Query<&TurretSectionBarrelRenderMesh, With<TurretSectionRotatorBarrelMarker>>,
) {
    let entity = add.entity;
    debug!(
        "Inserting render for TurretSectionRotatorBarrelMarker: {:?}",
        entity
    );

    let Ok(render_mesh) = q_barrel.get(entity) else {
        warn!(
            "TurretSectionRotatorBarrelMarker entity {:?} missing TurretSectionBarrelRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene) => {
            commands.entity(entity).insert((children![(
                Name::new("Render Turret Barrel"),
                SceneRoot(scene.clone()),
            ),],));
            return;
        }
        None => {
            let body_mat = materials.add(Color::srgb(0.2, 0.2, 0.5));
            let barrel_mat = materials.add(Color::srgb(0.2, 0.2, 0.7));
            let tip_mat = materials.add(Color::srgb(0.9, 0.2, 0.2));

            let body_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.3));
            let barrel_mesh = meshes.add(Cuboid::new(0.12, 0.12, 0.2));
            let tip_mesh = meshes.add(Cone::new(0.08, 0.18));

            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn((
                        Name::new("Render Turret Barrel"),
                        Transform::default(),
                        Visibility::Inherited,
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                Name::new("Turret Body"),
                                Transform::from_xyz(0.0, 0.0, -0.05),
                                Mesh3d(body_mesh.clone()),
                                MeshMaterial3d(body_mat.clone()),
                            ))
                            .with_children(|parent| {
                                parent
                                    .spawn((
                                        Name::new("Turret Barrel"),
                                        Transform::from_xyz(0.0, 0.0, -0.25),
                                        Mesh3d(barrel_mesh.clone()),
                                        MeshMaterial3d(barrel_mat.clone()),
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Name::new("Barrel Tip"),
                                            Transform::from_xyz(0.0, 0.0, -0.05).with_rotation(
                                                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                                            ),
                                            Mesh3d(tip_mesh.clone()),
                                            MeshMaterial3d(tip_mat.clone()),
                                        ));
                                    });
                            });
                    });
            });
        }
    }
}

// TODO: move this thing to the debug crate
mod debug {
    use super::*;

    pub struct DebugTurretSectionPlugin;

    impl Plugin for DebugTurretSectionPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                Update,
                (debug_draw_barrel_direction, debug_gizmos_turret_forward)
                    .in_set(TurretSectionPluginSet),
            );
        }
    }

    const DEBUG_LINE_LENGTH: f32 = 100.0;

    fn debug_draw_barrel_direction(
        q_barrel: Query<&GlobalTransform, With<TurretSectionRotatorBarrelMarker>>,
        mut gizmos: Gizmos,
    ) {
        for barrel_transform in &q_barrel {
            let barrel_pos = barrel_transform.translation();
            let barrel_dir = barrel_transform.forward();

            let line_length = DEBUG_LINE_LENGTH;
            let line_end = barrel_pos + barrel_dir * line_length;

            gizmos.line(barrel_pos, line_end, Color::srgb(1.0, 0.0, 0.0));
        }
    }

    fn debug_gizmos_turret_forward(
        mut gizmos: Gizmos,
        q_turret: Query<(&GlobalTransform, &TurretSectionTargetInput), With<TurretSectionMarker>>,
    ) {
        for (transform, target) in &q_turret {
            if let Some(target) = **target {
                let origin = transform.translation();
                let dir = (target - origin).normalize() * DEBUG_LINE_LENGTH;
                gizmos.line(origin, origin + dir, Color::srgb(0.9, 0.9, 0.1));
            }
        }
    }
}
