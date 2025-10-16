//! In this example, I want to demo how to use StableTorquePdController to rotate a spaceship to
//! follow the mouse cursor. The spaceship will rotate to face the mouse cursor when moved.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::Rng;

#[derive(Parser)]
#[command(name = "turret_target")]
#[command(version = "0.1")]
#[command(about = "Example for the section module", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();

    let mut app = new_gui_app();

    // Setup the scene with some entities, to have something to look at.
    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_scene, setup_turret, setup_simple_scene),
    );

    // Setup the input system to get input from the mouse and keyboard.
    app.add_input_context::<PlayerInputMarker>();
    app.add_observer(on_rotation_input);
    app.add_observer(on_rotation_input_completed);
    app.add_observer(on_combat_input_started);
    app.add_observer(on_combat_input_completed);

    app.insert_resource(SpaceshipControlMode::default());

    app.add_systems(
        Update,
        (
            update_chase_camera_input.before(ChaseCameraPluginSet),
            sync_random_orbit_state.after(SphereRandomOrbitPluginSet),
            update_turret_target_input.before(SpaceshipPluginSet),
        )
            .chain(),
    );

    app.add_systems(Update, sync_spaceship_control_mode);

    app.run();
}

fn setup_turret(mut commands: Commands) {
    commands.spawn((
        spaceship_root(SpaceshipConfig { ..default() }),
        children![(turret_section(TurretSectionConfig { ..default() }),),],
    ));
}

fn update_chase_camera_input(
    camera: Single<&mut ChaseCameraInput, With<ChaseCamera>>,
    spaceship: Single<&Transform, With<SpaceshipRootMarker>>,
    point_rotation: Single<&PointRotationOutput, With<SpaceshipRotationInputActiveMarker>>,
) {
    let mut camera_input = camera.into_inner();
    let spaceship_transform = spaceship.into_inner();
    let rotation = point_rotation.into_inner();

    camera_input.anchor_pos = spaceship_transform.translation;
    camera_input.achor_rot = **rotation;
}

fn sync_random_orbit_state(
    mut q_orbit: Query<
        (&mut Transform, &RandomSphereOrbitOutput),
        Changed<RandomSphereOrbitOutput>,
    >,
) {
    for (mut transform, output) in &mut q_orbit {
        transform.translation = **output;
    }
}

fn update_turret_target_input(
    target: Single<&GlobalTransform, With<PDCTurretTargetMarker>>,
    mut q_turret: Query<&mut TurretSectionTargetInput, With<TurretSectionMarker>>,
    mode: Res<SpaceshipControlMode>,
    point_rotation: Single<&PointRotationOutput, With<CombatRotationInputMarker>>,
    spaceship: Single<&GlobalTransform, With<SpaceshipRootMarker>>,
) {
    if matches!(*mode, SpaceshipControlMode::Combat) {
        let rotation = point_rotation.into_inner();
        let spaceship_transform = spaceship.into_inner();

        for mut turret in &mut q_turret {
            let forward = **rotation * -Vec3::Z;
            let position = spaceship_transform.translation();
            let distance = 100.0;

            **turret = Some(position + forward * distance);
        }
    } else {
        let target_transform = target.into_inner();

        for mut turret in &mut q_turret {
            **turret = Some(target_transform.translation());
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
enum SpaceshipControlMode {
    #[default]
    Normal,
    Combat,
}

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationInputActiveMarker;

fn sync_spaceship_control_mode(
    mut commands: Commands,
    mode: Res<SpaceshipControlMode>,
    spaceship_input_rotation: Single<
        (Entity, &PointRotationOutput),
        With<SpaceshipRotationInputMarker>,
    >,
    spaceship_input_combat: Single<Entity, With<CombatRotationInputMarker>>,
    camera: Single<Entity, With<ChaseCamera>>,
) {
    if !mode.is_changed() {
        return;
    }

    let (spaceship_input_rotation, point_rotation) = spaceship_input_rotation.into_inner();
    let spaceship_input_combat = spaceship_input_combat.into_inner();
    let camera = camera.into_inner();

    match *mode {
        SpaceshipControlMode::Normal => {
            commands
                .entity(spaceship_input_rotation)
                .insert(SpaceshipRotationInputActiveMarker);
            commands
                .entity(spaceship_input_combat)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands.entity(camera).insert(ChaseCamera {
                offset: Vec3::new(0.0, 5.0, -20.0),
                focus_offset: Vec3::new(0.0, 0.0, 20.0),
                ..default()
            });
        }
        SpaceshipControlMode::Combat => {
            commands
                .entity(spaceship_input_rotation)
                .remove::<SpaceshipRotationInputActiveMarker>();
            commands
                .entity(spaceship_input_combat)
                .insert(PointRotation {
                    initial_rotation: **point_rotation,
                })
                .insert(SpaceshipRotationInputActiveMarker);
            commands.entity(camera).insert(ChaseCamera {
                offset: Vec3::new(0.0, 1.0, -10.0),
                focus_offset: Vec3::new(0.0, 0.0, 50.0),
                ..default()
            });
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct PDCTurretTargetMarker;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    // Spawn a player input controller entity to hold the input from the player
    commands.spawn((
        Name::new("Player Input Controller"),
        Transform::default(),
        GlobalTransform::default(),
        PlayerInputMarker,
        actions!(
            PlayerInputMarker[
                (
                    Action::<CameraInputRotate>::new(),
                    Bindings::spawn((
                        // Bevy requires single entities to be wrapped in `Spawn`.
                        // You can attach modifiers to individual bindings as well.
                        Spawn((Binding::mouse_motion(), Scale::splat(0.001), Negate::all())),
                        Axial::right_stick().with((Scale::splat(2.0), Negate::none())),
                    )),
                ),
                (
                    Action::<ThrusterInput>::new(),
                    bindings![KeyCode::KeyW, GamepadButton::RightTrigger],
                ),
                (
                    Action::<CombatInput>::new(),
                    bindings![MouseButton::Right],
                ),
            ]
        ),
    ));

    // Spawn a RotationInput to consume the mouse movement and will be used to rotate the spaceship
    commands.spawn((
        Name::new("Spaceship Rotation Input"),
        SpaceshipRotationInputMarker,
        SpaceshipRotationInputActiveMarker,
        PointRotation::default(),
    ));

    // Spawn a RotationInput to consume the mouse movement and will be used to rotate the combat
    commands.spawn((
        Name::new("Combat Rotation Input"),
        CombatRotationInputMarker,
        PointRotation::default(),
    ));

    // Spawn a 3D camera with a chase camera component
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        ChaseCamera::default(),
        SkyboxConfig {
            cubemap: game_assets.cubemap.clone(),
            brightness: 1000.0,
        },
    ));

    // Spawn a target entity to visualize the target rotation
    commands.spawn((
        Name::new("Turret Target"),
        PDCTurretTargetMarker,
        RandomSphereOrbit {
            radius: 5.0,
            angular_speed: 5.0,
            center: Vec3::ZERO,
            ..default()
        },
        Transform::default(),
        Visibility::Visible,
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
    ));
}

#[derive(Component, Debug, Clone)]
struct PlayerInputMarker;

#[derive(Component, Debug, Clone)]
struct SpaceshipRotationInputMarker;

#[derive(Component, Debug, Clone)]
struct CombatRotationInputMarker;

#[derive(InputAction)]
#[action_output(Vec2)]
struct CameraInputRotate;

#[derive(InputAction)]
#[action_output(bool)]
struct ThrusterInput;

#[derive(InputAction)]
#[action_output(bool)]
struct CombatInput;

fn on_rotation_input(
    fire: On<Fire<CameraInputRotate>>,
    mut q_input: Query<&mut PointRotationInput, With<SpaceshipRotationInputActiveMarker>>,
) {
    for mut input in &mut q_input {
        **input = fire.value;
    }
}

fn on_rotation_input_completed(
    _: On<Complete<CameraInputRotate>>,
    mut q_input: Query<&mut PointRotationInput>,
) {
    for mut input in &mut q_input {
        **input = Vec2::ZERO;
    }
}

fn on_combat_input_started(_: On<Start<CombatInput>>, mut mode: ResMut<SpaceshipControlMode>) {
    *mode = SpaceshipControlMode::Combat;
}

fn on_combat_input_completed(_: On<Complete<CombatInput>>, mut mode: ResMut<SpaceshipControlMode>) {
    *mode = SpaceshipControlMode::Normal;
}

pub fn setup_simple_scene(
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
