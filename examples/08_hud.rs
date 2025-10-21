use avian3d::prelude::*;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "08_hud")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how the hud will work in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();

    app.add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>,
    >::default());

    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_spaceship, setup_hud, setup_camera, setup_simple_scene),
    );

    app.add_systems(Update, on_thruster_input);

    app.add_systems(
        Update,
        (
            update_velocity_hud_input.before(SphereOrbitPluginSet),
            sync_orbit_state.after(SphereOrbitPluginSet),
            direction_shader_update_system,
        ),
    );

    app.run();
}

#[derive(Component, Debug, Clone)]
struct VelocityHudMarker;

fn setup_hud(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut direction_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>,
    >,
) {
    commands.spawn((
        Name::new("HUD"),
        VelocityHudMarker,
        DirectionalSphereOrbit {
            radius: 10.0,
            ..default()
        },
        Transform::default(),
        Visibility::Visible,
        children![(
            Name::new("Velocity Indicator"),
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
        )],
    ));
}

fn update_velocity_hud_input(
    hud: Single<&mut DirectionalSphereOrbitInput, With<VelocityHudMarker>>,
    spaceship: Single<&LinearVelocity, With<SpaceshipRootMarker>>,
) {
    let spaceship_velocity = spaceship.into_inner();
    let spaceship_dir = spaceship_velocity.normalize_or_zero();
    if spaceship_dir == Vec3::ZERO {
        return;
    }

    let mut hud_input = hud.into_inner();

    **hud_input = spaceship_dir;
}

fn sync_orbit_state(
    mut q_orbit: Query<
        (&mut Transform, &DirectionalSphereOrbitOutput),
        Changed<DirectionalSphereOrbitOutput>,
    >,
    spaceship: Single<&GlobalTransform, With<SpaceshipRootMarker>>,
) {
    let spaceship_transform = spaceship.into_inner();
    let spaceship_origin = spaceship_transform.translation();

    for (mut transform, output) in &mut q_orbit {
        let dir = **output;
        transform.translation = spaceship_origin + dir;
        transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, dir.normalize_or_zero());
    }
}

fn direction_shader_update_system(
    spaceship: Single<&LinearVelocity, With<SpaceshipRootMarker>>,
    q_hud: Query<Entity, With<VelocityHudMarker>>,
    q_render: Query<
        (&MeshMaterial3d<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>, &ChildOf),
    >,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>>,
) {
    let spaceship_velocity = spaceship.into_inner();
    let magnitude = spaceship_velocity.length();

    for (material, &ChildOf(parent)) in &q_render {
        let Ok(_) = q_hud.get(parent) else {
            warn!("VelocityHudMarker's parent is not HudMarker");
            continue;
        };

        let Some(material) = materials.get_mut(&**material) else {
            warn!("Failed to get material for VelocityHudMarker");
            continue;
        };

        material.extension.magnitude_input = magnitude;
    }
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

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect)]
struct ThrusterInputKey(KeyCode);

fn on_thruster_input(
    mut q_input: Query<(&mut ThrusterSectionInput, &ThrusterInputKey)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut input, key) in &mut q_input {
        if keys.pressed(key.0) {
            **input = 1.0;
        } else {
            **input = 0.0;
        }
    }
}

fn setup_spaceship(mut commands: Commands) {
    let entity = commands
        .spawn((spaceship_root(SpaceshipConfig { ..default() }),))
        .id();

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            thruster_section(ThrusterSectionConfig {
                magnitude: 1.0,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            }),
            ThrusterInputKey(KeyCode::Digit1),
        ));
    });
}

fn setup_camera(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        SkyboxConfig {
            cubemap: game_assets.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
}

fn setup_simple_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_2,
            0.0,
            0.0,
        )),
        GlobalTransform::default(),
    ));

    for i in 0..20 {
        let pos = Vec3::new(
            rng.random_range(-100.0..100.0),
            rng.random_range(-20.0..20.0),
            rng.random_range(-100.0..100.0),
        );
        let radius = rng.random_range(2.0..6.0);
        let color = Color::srgb(
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
        );

        commands.spawn((
            Name::new(format!("Planet {}", i)),
            Transform::from_translation(pos),
            GlobalTransform::default(),
            Mesh3d(meshes.add(Sphere::new(radius))),
            MeshMaterial3d(materials.add(color)),
            Collider::sphere(radius),
            RigidBody::Static,
        ));
    }

    for i in 0..40 {
        let pos = Vec3::new(
            rng.random_range(-120.0..120.0),
            rng.random_range(-30.0..30.0),
            rng.random_range(-120.0..120.0),
        );
        let size = rng.random_range(0.5..1.0);
        let color = Color::srgb(
            rng.random_range(0.6..1.0),
            rng.random_range(0.6..1.0),
            rng.random_range(0.0..0.6),
        );

        commands.spawn((
            Name::new(format!("Satellite {}", i)),
            Transform::from_translation(pos),
            GlobalTransform::default(),
            Mesh3d(meshes.add(Cuboid::new(size, size, size))),
            MeshMaterial3d(materials.add(color)),
            Collider::cuboid(size, size, size),
            ColliderDensity(1.0),
            RigidBody::Dynamic,
        ));
    }
}
