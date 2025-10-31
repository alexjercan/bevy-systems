use avian3d::prelude::*;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use bevy_common_systems::prelude::*;

use crate::prelude::SpaceshipSystems;

pub mod prelude {
    pub use super::{
        velocity_hud, VelocityHudConfig, VelocityHudIndicatorMarker, VelocityHudMarker,
        VelocityHudPlugin, VelocityHudTargetEntity,
    };
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct VelocityHudMarker;

#[derive(Component, Debug, Clone, Reflect)]
pub struct VelocityHudIndicatorMarker;

#[derive(Clone, Debug)]
pub struct VelocityHudConfig {
    pub radius: f32,
    pub target: Option<Entity>,
}

impl Default for VelocityHudConfig {
    fn default() -> Self {
        Self {
            radius: 5.0,
            target: None,
        }
    }
}

pub fn velocity_hud(config: VelocityHudConfig) -> impl Bundle {
    debug!("velocity_hud: config {:?}", config);

    (
        Name::new("VelocityHUD"),
        VelocityHudMarker,
        VelocityHudTargetEntity(config.target),
        DirectionalSphereOrbit {
            radius: config.radius,
            ..default()
        },
        Transform::default(),
        Visibility::Visible,
    )
}

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
pub struct VelocityHudTargetEntity(Option<Entity>);

#[derive(Default)]
pub struct VelocityHudPlugin;

impl Plugin for VelocityHudPlugin {
    fn build(&self, app: &mut App) {
        debug!("VelocityHudPlugin: build");

        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>,
        >::default());

        app.add_observer(insert_velocity_hud_indicator_system);

        app.add_systems(
            Update,
            (
                update_velocity_hud_input,
                sync_orbit_state,
                direction_shader_update_system,
            )
                .in_set(SpaceshipSystems::Hud),
        );
    }
}

fn update_velocity_hud_input(
    mut q_hud: Query<
        (
            Entity,
            &mut DirectionalSphereOrbitInput,
            &VelocityHudTargetEntity,
        ),
        With<VelocityHudMarker>,
    >,
    q_target: Query<&LinearVelocity>,
) {
    for (entity, mut hud_input, target) in q_hud.iter_mut() {
        let Some(target) = **target else {
            warn!(
                "update_velocity_hud_input: entity {:?} has no target entity set",
                entity
            );
            continue;
        };

        let Ok(velocity) = q_target.get(target) else {
            warn!(
                "update_velocity_hud_input: entity {:?} not found in q_target",
                target
            );
            continue;
        };

        let dir = velocity.0.normalize_or_zero();
        **hud_input = dir;
    }
}

fn sync_orbit_state(
    mut q_orbit: Query<
        (
            Entity,
            &mut Transform,
            &DirectionalSphereOrbitOutput,
            &VelocityHudTargetEntity,
        ),
        (
            Changed<DirectionalSphereOrbitOutput>,
            With<VelocityHudMarker>,
        ),
    >,
    q_target: Query<&Transform, Without<VelocityHudMarker>>,
) {
    for (entity, mut transform, output, target) in &mut q_orbit {
        let Some(target) = **target else {
            warn!(
                "sync_orbit_state: entity {:?} has no target entity set",
                entity
            );
            continue;
        };

        let Ok(target_transform) = q_target.get(target) else {
            warn!(
                "sync_orbit_state: entity {:?} not found in q_target",
                target
            );
            continue;
        };

        let origin = target_transform.translation;
        let dir = **output;
        transform.translation = origin + dir;
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, dir.normalize_or_zero());
    }
}

fn direction_shader_update_system(
    q_target: Query<&LinearVelocity>,
    q_hud: Query<(Entity, &VelocityHudTargetEntity), With<VelocityHudMarker>>,
    q_render: Query<(
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>,
        &ChildOf,
    )>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>>,
) {
    for (material, &ChildOf(parent)) in &q_render {
        let Ok((_, target)) = q_hud.get(parent) else {
            warn!(
                "direction_shader_update_system: parent entity {:?} not found in q_hud",
                parent
            );
            continue;
        };

        let Some(target) = **target else {
            warn!(
                "direction_shader_update_system: entity {:?} has no target entity set",
                parent
            );
            continue;
        };

        let Ok(velocity) = q_target.get(target) else {
            warn!(
                "direction_shader_update_system: entity {:?} not found in q_target",
                target
            );
            continue;
        };
        let magnitude = velocity.length() / 100.0;

        let Some(material) = materials.get_mut(&**material) else {
            warn!(
                "direction_shader_update_system: material for entity {:?} not found",
                parent
            );
            continue;
        };

        material.extension.magnitude_input = magnitude;
    }
}

fn insert_velocity_hud_indicator_system(
    add: On<Add, VelocityHudMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut direction_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>,
    >,
) {
    let entity = add.entity;
    trace!("insert_velocity_hud_indicator_system: entity {:?}", entity);

    commands.entity(entity).insert(children![(
        Name::new("VelocityHUD Indicator"),
        VelocityHudIndicatorMarker,
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Mesh3d(meshes.add(Cone::new(0.2, 0.1))),
        MeshMaterial3d(
            direction_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    perceptual_roughness: 1.0,
                    metallic: 0.0,
                    ..default()
                },
                extension: DirectionMagnitudeMaterial::default()
                    .with_max_height(1.0)
                    .with_radius(0.2),
            }),
        ),
    ),]);
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct DirectionMagnitudeMaterial {
    #[uniform(100)]
    pub magnitude_input: f32,
    #[uniform(100)]
    pub radius: f32,
    #[uniform(100)]
    pub max_height: f32,
    #[cfg(target_arch = "wasm32")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
}

impl DirectionMagnitudeMaterial {
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }
}

impl MaterialExtension for DirectionMagnitudeMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/directional_magnitude.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/directional_magnitude.wgsl".into()
    }
}
