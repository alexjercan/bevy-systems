use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "nova_protocol")]
#[command(version = "0.0.2")]
#[command(about = "Simple spaceship editor scene where you can build custom ships", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();

    app.init_state::<SceneState>();

    app.add_systems(OnEnter(GameStates::Playing), setup_status_ui);

    // We start in the editor state
    app.add_systems(
        OnEnter(GameStates::Playing),
        |mut state: ResMut<NextState<SceneState>>| {
            state.set(SceneState::Editor);
        },
    );
    // On F1 we switch to editor
    app.add_systems(
        Update,
        switch_scene_editor.run_if(in_state(SceneState::Simulation)),
    );

    // TODO: Will move Editor/Simulation to gameplay once they are fully tested and functional

    // Editor
    app.add_plugins(editor::editor_plugin);

    // Simulation
    app.add_plugins(simulation::simulation_plugin);

    app.add_systems(
        Update,
        (on_thruster_input, on_projectile_input).run_if(in_state(SceneState::Simulation)),
    );
    app.add_systems(
        OnExit(SceneState::Simulation),
        |mut q_thruster: Query<&mut ThrusterSectionInput, With<ThrusterInputKey>>| {
            for mut input in &mut q_thruster {
                **input = 0.0;
            }
        },
    );
    app.add_systems(
        OnExit(SceneState::Simulation),
        |mut commands: Commands, q_fragment: Query<Entity, With<FragmentMeshMarker>>| {
            for fragment in &q_fragment {
                commands.entity(fragment).despawn();
            }
        },
    );

    app.run();
}

fn setup_status_ui(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn(status_bar(StatusBarRootConfig::default()));
    commands.spawn(status_bar_item(StatusBarItemConfig {
        icon: Some(game_assets.fps_icon.clone()),
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        label: "fps".to_string(),
    }));
    commands.spawn(status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_version_value_fn(env!("CARGO_PKG_VERSION").to_string()),
        color_fn: status_version_color_fn(),
        label: "version".to_string(),
    }));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum SceneState {
    #[default]
    None,
    Editor,
    Simulation,
}

fn switch_scene_editor(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<SceneState>>) {
    if keys.just_pressed(KeyCode::F1) {
        state.set(SceneState::Editor);
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

fn on_projectile_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    q_spawner: Query<Entity, With<ProjectileSpawnerMarker<BulletProjectileConfig>>>,
) {
    for spawner_entity in &q_spawner {
        if mouse.pressed(MouseButton::Left) {
            commands.trigger(SpawnProjectile {
                entity: spawner_entity,
            });
        }
    }
}

mod simulation {
    use avian3d::prelude::*;
    use bevy::{
        pbr::{ExtendedMaterial, MaterialExtension},
        prelude::*,
        render::render_resource::AsBindGroup,
        shader::ShaderRef,
        window::{CursorGrabMode, CursorOptions, PrimaryWindow},
    };
    use bevy_enhanced_input::prelude::*;
    use nova_protocol::prelude::*;
    use rand::prelude::*;

    pub fn simulation_plugin(app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>,
        >::default());

        app.add_systems(
            OnEnter(super::SceneState::Simulation),
            (
                setup_scene,
                setup_hud,
                setup_simple_scene,
                setup_grab_cursor,
            ),
        );

        // Setup the input system to get input from the mouse and keyboard.
        app.add_input_context::<PlayerInputMarker>();
        app.add_observer(on_rotation_input);
        app.add_observer(on_rotation_input_completed);
        app.add_observer(on_free_mode_input_started);
        app.add_observer(on_free_mode_input_completed);
        app.add_observer(on_combat_input_started);
        app.add_observer(on_combat_input_completed);

        // Spaceship Control Mode for the Camera/Spaceship
        app.insert_resource(SpaceshipControlMode::default());
        app.add_systems(Update, sync_spaceship_control_mode);

        app.add_systems(
            Update,
            (
                update_chase_camera_input.before(ChaseCameraPluginSet),
                update_velocity_hud_input.before(DirectionalSphereOrbitPluginSet),
                (
                    update_text_hud,
                    update_spaceship_target_rotation_torque,
                    update_turret_target_input,
                )
                    .before(SpaceshipPluginSet),
                sync_orbit_state.after(DirectionalSphereOrbitPluginSet),
                direction_shader_update_system,
            )
                .chain(),
        );
    }

    #[derive(Component, Debug, Clone)]
    struct VelocityHudMarker;

    #[derive(Component, Debug, Clone)]
    struct HealthHudMarker;

    fn setup_hud(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut direction_materials: ResMut<
            Assets<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>,
        >,
    ) {
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("HUD"),
            VelocityHudMarker,
            DirectionalSphereOrbit {
                radius: 5.0,
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

        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("Health HUD"),
            HealthHudMarker,
            Text::new("Health: 100%"),
            TextShadow::default(),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(5),
                right: px(5),
                ..default()
            },
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

    fn update_text_hud(
        spaceship: Single<&Health, With<SpaceshipRootMarker>>,
        mut q_hud: Query<&mut Text, With<HealthHudMarker>>,
    ) {
        let health = spaceship.into_inner();
        let health_percent = (health.current / health.max * 100.0).round();

        for mut text in &mut q_hud {
            text.0 = format!("Health: {}%", health_percent);
        }
    }

    fn sync_orbit_state(
        mut q_orbit: Query<
            (&mut Transform, &DirectionalSphereOrbitOutput),
            (
                Changed<DirectionalSphereOrbitOutput>,
                Without<SpaceshipRootMarker>,
            ),
        >,
        spaceship: Single<&Transform, With<SpaceshipRootMarker>>,
    ) {
        let spaceship_transform = spaceship.into_inner();
        let spaceship_origin = spaceship_transform.translation;

        for (mut transform, output) in &mut q_orbit {
            let dir = **output;
            transform.translation = spaceship_origin + dir;
            transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, dir.normalize_or_zero());
        }
    }

    fn direction_shader_update_system(
        spaceship: Single<&LinearVelocity, With<SpaceshipRootMarker>>,
        q_hud: Query<Entity, With<VelocityHudMarker>>,
        q_render: Query<(
            &MeshMaterial3d<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>,
            &ChildOf,
        )>,
        mut materials: ResMut<
            Assets<ExtendedMaterial<StandardMaterial, DirectionMagnitudeMaterial>>,
        >,
    ) {
        let spaceship_velocity = spaceship.into_inner();
        let magnitude = spaceship_velocity.length() / 100.0;

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

    fn update_spaceship_target_rotation_torque(
        point_rotation: Single<&PointRotationOutput, With<SpaceshipRotationInputMarker>>,
        controller: Single<&mut ControllerSectionRotationInput, With<ControllerSectionMarker>>,
    ) {
        let rotation = point_rotation.into_inner();
        let mut controller_target = controller.into_inner();
        **controller_target = **rotation;
    }

    #[derive(Component, Clone, Copy, Debug, Reflect)]
    struct PDCTurretTargetMarker;

    fn update_turret_target_input(
        target: Option<Single<&GlobalTransform, With<PDCTurretTargetMarker>>>,
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
            let Some(target) = target else {
                return;
            };

            let target_transform = target.into_inner();

            for mut turret in &mut q_turret {
                **turret = Some(target_transform.translation());
            }
        }
    }

    fn setup_scene(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        game_assets: Res<GameAssets>,
    ) {
        // Spawn a player input controller entity to hold the input from the player
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
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
                        Action::<FreeLookInput>::new(),
                        bindings![KeyCode::AltLeft, GamepadButton::LeftTrigger],
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
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("Spaceship Rotation Input"),
            SpaceshipRotationInputMarker,
            SpaceshipRotationInputActiveMarker,
            PointRotation::default(),
        ));

        // Spawn a RotationInput to consume the mouse movement and will be used to rotate the free look
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("FreeLook Rotation Input"),
            FreeLookRotationInputMarker,
            PointRotation::default(),
        ));

        // Spawn a RotationInput to consume the mouse movement and will be used to rotate the combat
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("Combat Rotation Input"),
            CombatRotationInputMarker,
            PointRotation::default(),
        ));

        // Spawn a 3D camera with a chase camera component
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("Chase Camera"),
            Camera3d::default(),
            ChaseCamera::default(),
            Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            SkyboxConfig {
                cubemap: game_assets.cubemap.clone(),
                brightness: 1000.0,
            },
        ));

        // Spawn a target entity to visualize the target rotation
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("Turret Target"),
            PDCTurretTargetMarker,
            Transform::from_xyz(0.0, 0.0, -500.0),
            Visibility::Visible,
            Mesh3d(meshes.add(Cuboid::new(3.0, 3.0, 3.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.2))),
            Collider::cuboid(3.0, 3.0, 3.0),
            RigidBody::Static,
        ));
    }

    pub fn setup_simple_scene(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let mut rng = rand::rng();

        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
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

            let planet = commands
                .spawn((
                    DespawnOnExit(super::SceneState::Simulation),
                    Name::new(format!("Planet {}", i)),
                    Transform::from_translation(pos),
                    GlobalTransform::default(),
                    Mesh3d(meshes.add(Sphere::new(radius))),
                    MeshMaterial3d(materials.add(color)),
                    Collider::sphere(radius),
                    RigidBody::Static,
                    Health::new(100.0),
                    CollisionDamageMarker,
                ))
                .id();

            commands.entity(planet).insert(ExplodeOnDestroy {
                mesh_entity: Some(planet),
                fragment_count: 8,
                ..default()
            });
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

            let satellite = commands
                .spawn((
                    DespawnOnExit(super::SceneState::Simulation),
                    Name::new(format!("Satellite {}", i)),
                    Transform::from_translation(pos),
                    GlobalTransform::default(),
                    Mesh3d(meshes.add(Cuboid::new(size, size, size))),
                    MeshMaterial3d(materials.add(color)),
                    Collider::cuboid(size, size, size),
                    ColliderDensity(1.0),
                    RigidBody::Dynamic,
                    Health::new(100.0),
                    CollisionDamageMarker,
                ))
                .id();

            commands.entity(satellite).insert(ExplodeOnDestroy {
                mesh_entity: Some(satellite),
                fragment_count: 4,
                ..default()
            });
        }
    }

    fn setup_grab_cursor(primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
        let mut primary_cursor_options = primary_cursor_options.into_inner();
        primary_cursor_options.grab_mode = CursorGrabMode::Locked;
        primary_cursor_options.visible = false;
    }

    #[derive(Resource, Default, Clone, Debug)]
    enum SpaceshipControlMode {
        #[default]
        Normal,
        FreeLook,
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
        spaceship_input_free_look: Single<Entity, With<FreeLookRotationInputMarker>>,
        spaceship_input_combat: Single<Entity, With<CombatRotationInputMarker>>,
        camera: Single<Entity, With<ChaseCamera>>,
    ) {
        if !mode.is_changed() {
            return;
        }

        let (spaceship_input_rotation, point_rotation) = spaceship_input_rotation.into_inner();
        let spaceship_input_free_look = spaceship_input_free_look.into_inner();
        let spaceship_input_combat = spaceship_input_combat.into_inner();
        let camera = camera.into_inner();

        match *mode {
            SpaceshipControlMode::Normal => {
                commands
                    .entity(spaceship_input_rotation)
                    .insert(SpaceshipRotationInputActiveMarker);
                commands
                    .entity(spaceship_input_free_look)
                    .remove::<SpaceshipRotationInputActiveMarker>();
                commands
                    .entity(spaceship_input_combat)
                    .remove::<SpaceshipRotationInputActiveMarker>();
                commands.entity(camera).insert(ChaseCamera {
                    offset: Vec3::new(0.0, 5.0, -20.0),
                    focus_offset: Vec3::new(0.0, 0.0, 20.0),
                    ..default()
                });
            }
            SpaceshipControlMode::FreeLook => {
                commands
                    .entity(spaceship_input_rotation)
                    .remove::<SpaceshipRotationInputActiveMarker>();
                commands
                    .entity(spaceship_input_free_look)
                    .insert(PointRotation {
                        initial_rotation: **point_rotation,
                    })
                    .insert(SpaceshipRotationInputActiveMarker);
                commands
                    .entity(spaceship_input_combat)
                    .remove::<SpaceshipRotationInputActiveMarker>();
                commands.entity(camera).insert(ChaseCamera {
                    offset: Vec3::new(0.0, 10.0, -30.0),
                    focus_offset: Vec3::new(0.0, 0.0, 0.0),
                    ..default()
                });
            }
            SpaceshipControlMode::Combat => {
                commands
                    .entity(spaceship_input_rotation)
                    .remove::<SpaceshipRotationInputActiveMarker>();
                commands
                    .entity(spaceship_input_free_look)
                    .remove::<SpaceshipRotationInputActiveMarker>();
                commands
                    .entity(spaceship_input_combat)
                    .insert(PointRotation {
                        initial_rotation: **point_rotation,
                    })
                    .insert(SpaceshipRotationInputActiveMarker);
                commands.entity(camera).insert(ChaseCamera {
                    offset: Vec3::new(0.0, 5.0, -10.0),
                    focus_offset: Vec3::new(0.0, 0.0, 50.0),
                    ..default()
                });
            }
        }
    }

    #[derive(Component, Debug, Clone)]
    struct PlayerInputMarker;

    #[derive(Component, Debug, Clone)]
    struct SpaceshipRotationInputMarker;

    #[derive(Component, Debug, Clone)]
    struct FreeLookRotationInputMarker;

    #[derive(Component, Debug, Clone)]
    struct CombatRotationInputMarker;

    #[derive(InputAction)]
    #[action_output(Vec2)]
    struct CameraInputRotate;

    #[derive(InputAction)]
    #[action_output(bool)]
    struct FreeLookInput;

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

    fn on_free_mode_input_started(
        _: On<Start<FreeLookInput>>,
        mut mode: ResMut<SpaceshipControlMode>,
    ) {
        *mode = SpaceshipControlMode::FreeLook;
    }

    fn on_free_mode_input_completed(
        _: On<Complete<FreeLookInput>>,
        mut mode: ResMut<SpaceshipControlMode>,
    ) {
        *mode = SpaceshipControlMode::Normal;
    }

    fn on_combat_input_started(_: On<Start<CombatInput>>, mut mode: ResMut<SpaceshipControlMode>) {
        *mode = SpaceshipControlMode::Combat;
    }

    fn on_combat_input_completed(
        _: On<Complete<CombatInput>>,
        mut mode: ResMut<SpaceshipControlMode>,
    ) {
        *mode = SpaceshipControlMode::Normal;
    }
}

mod editor {
    // https://github.com/bevyengine/bevy/blob/release-0.17.2/examples/ui/standard_widgets_observers.rs

    use bevy::{
        picking::{hover::Hovered, pointer::PointerInteraction},
        prelude::*,
        reflect::Is,
        ui::{InteractionDisabled, Pressed},
        ui_widgets::{observe, Activate, Button},
        window::{CursorGrabMode, CursorOptions, PrimaryWindow},
    };
    use nova_protocol::prelude::*;

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
    const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);

    const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

    const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

    #[derive(Resource, Default, Debug, Component, PartialEq, Eq, Clone, Copy, Reflect)]
    enum SectionChoice {
        #[default]
        None,
        HullSection,
        ThrusterSection,
        TurretSection,
        Delete,
    }

    pub fn editor_plugin(app: &mut App) {
        app.insert_resource(SectionChoice::None);

        app.add_systems(
            OnEnter(super::SceneState::Editor),
            (reset_spaceship, setup_editor_scene, setup_grab_cursor),
        );
        app.add_systems(
            Update,
            lock_on_left_click.run_if(in_state(super::SceneState::Editor)),
        );

        app.add_observer(button_on_interaction::<Add, Pressed>)
            .add_observer(button_on_interaction::<Remove, Pressed>)
            .add_observer(button_on_interaction::<Add, InteractionDisabled>)
            .add_observer(button_on_interaction::<Remove, InteractionDisabled>)
            .add_observer(button_on_interaction::<Insert, Hovered>);

        app.add_observer(on_add_selected)
            .add_observer(on_remove_selected);
        app.add_observer(button_on_setting::<SectionChoice>);

        app.add_observer(on_click_spaceship_section)
            .add_observer(on_hover_spaceship_section)
            .add_observer(on_move_spaceship_section)
            .add_observer(on_out_spaceship_section);

        app.add_systems(
            OnExit(super::SceneState::Editor),
            |mut selection: ResMut<SectionChoice>| {
                *selection = SectionChoice::None;
            },
        );
    }

    fn lock_on_left_click(
        primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
        mouse: Res<ButtonInput<MouseButton>>,
    ) {
        // TODO: Not for UI
        if mouse.just_pressed(MouseButton::Right) {
            let mut primary_cursor_options = primary_cursor_options.into_inner();
            primary_cursor_options.grab_mode = CursorGrabMode::Locked;
            primary_cursor_options.visible = false;
        } else if mouse.just_released(MouseButton::Right) {
            let mut primary_cursor_options = primary_cursor_options.into_inner();
            primary_cursor_options.grab_mode = CursorGrabMode::None;
            primary_cursor_options.visible = true;
        }
    }

    #[derive(Component)]
    struct SelectedOption;

    #[derive(Component)]
    struct EditorButton;

    fn button_on_interaction<E: EntityEvent, C: Component>(
        event: On<E, C>,
        mut q_button: Query<
            (
                &Hovered,
                Has<InteractionDisabled>,
                Has<Pressed>,
                Has<SelectedOption>,
                &mut BackgroundColor,
                &mut BorderColor,
                &Children,
            ),
            With<EditorButton>,
        >,
    ) {
        if let Ok((hovered, disabled, pressed, selected, mut color, mut border_color, children)) =
            q_button.get_mut(event.event_target())
        {
            if children.is_empty() {
                return;
            }
            if selected {
                *color = HOVERED_PRESSED_BUTTON.into();
                border_color.set_all(BORDER_COLOR_ACTIVE);
                return;
            }

            let hovered = hovered.get();
            let pressed = pressed && !(E::is::<Remove>() && C::is::<Pressed>());
            let disabled = disabled && !(E::is::<Remove>() && C::is::<InteractionDisabled>());
            match (disabled, hovered, pressed) {
                (true, _, _) => {
                    *color = NORMAL_BUTTON.into();
                    *border_color = BORDER_COLOR_INACTIVE.into();
                }

                (false, true, true) => {
                    *color = HOVERED_PRESSED_BUTTON.into();
                    border_color.set_all(BORDER_COLOR_ACTIVE);
                }

                (false, true, false) => {
                    *color = HOVERED_BUTTON.into();
                    border_color.set_all(BORDER_COLOR_ACTIVE);
                }

                (false, false, _) => {
                    *color = NORMAL_BUTTON.into();
                    *border_color = BORDER_COLOR_INACTIVE.into();
                }
            }
        }
    }

    fn button_on_setting<T: Resource + Component + PartialEq + Copy>(
        event: On<Add, Pressed>,
        mut commands: Commands,
        selected: Option<Single<Entity, (With<T>, With<SelectedOption>)>>,
        q_t: Query<(Entity, &T), (Without<SelectedOption>, With<EditorButton>)>,
        mut setting: ResMut<T>,
    ) {
        let Ok((entity, t)) = q_t.get(event.event_target()) else {
            return;
        };

        if *setting != *t {
            if let Some(previous) = selected {
                commands
                    .entity(previous.into_inner())
                    .remove::<SelectedOption>();
            }
            commands.entity(entity).insert(SelectedOption);
            *setting = *t;
        }
    }

    fn on_add_selected(
        add: On<Add, SelectedOption>,
        mut q_color: Query<&mut BackgroundColor, (With<SelectedOption>, With<EditorButton>)>,
    ) {
        if let Ok(mut color) = q_color.get_mut(add.event_target()) {
            *color = PRESSED_BUTTON.into();
        }
    }

    fn on_remove_selected(
        remove: On<Remove, SelectedOption>,
        mut q_color: Query<&mut BackgroundColor, With<EditorButton>>,
    ) {
        if let Ok(mut color) = q_color.get_mut(remove.event_target()) {
            *color = NORMAL_BUTTON.into();
        }
    }

    fn button(text: &str) -> impl Bundle {
        (
            Node {
                width: percent(80),
                min_height: px(40),
                margin: UiRect::all(px(20)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            EditorButton,
            Button,
            Hovered::default(),
            BorderColor::all(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new(text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )],
        )
    }

    fn reset_spaceship(
        mut commands: Commands,
        spaceship: Single<(Entity, &Children), With<SpaceshipRootMarker>>,
    ) {
        let (spaceship, children) = spaceship.into_inner();
        commands
            .spawn(spaceship_root(SpaceshipConfig { ..default() }))
            .add_children(children);
        commands
            .entity(spaceship)
            .remove_children(children)
            .despawn();
    }

    fn setup_editor_scene(mut commands: Commands, game_assets: Res<GameAssets>) {
        commands.spawn((
            DespawnOnExit(super::SceneState::Editor),
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

        commands.spawn((
            DespawnOnExit(super::SceneState::Editor),
            Name::new("WASD Camera"),
            Camera3d::default(),
            WASDCameraController,
            Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            SkyboxConfig {
                cubemap: game_assets.cubemap.clone(),
                brightness: 1000.0,
            },
        ));

        commands.spawn((
            DespawnOnExit(super::SceneState::Editor),
            Name::new("Editor Main Menu"),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            children![(
                Name::new("Menu Container"),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    height: percent(80),
                    width: px(400),
                    margin: UiRect::all(px(50)),
                    padding: UiRect::all(px(0)).with_top(px(20)).with_bottom(px(20)),
                    ..default()
                },
                BackgroundColor(BACKGROUND_COLOR),
                children![
                    (
                        Name::new("Title"),
                        Text::new("Spaceship Editor"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node { ..default() },
                    ),
                    (
                        Name::new("Separator 1"),
                        Node {
                            width: percent(80),
                            height: px(2),
                            margin: UiRect::all(px(10)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    ),
                    (
                        Name::new("Create New Spaceship Button V1"),
                        button("Create New Spaceship V1"),
                        observe(create_new_spaceship),
                    ),
                    (
                        Name::new("Create New Spaceship Button V2"),
                        button("Create New Spaceship V2"),
                        observe(create_new_spaceship_with_controller),
                    ),
                    (
                        Name::new("Separator 2"),
                        Node {
                            width: percent(80),
                            height: px(2),
                            margin: UiRect::all(px(10)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    ),
                    sections(),
                    (
                        Name::new("Separator 3"),
                        Node {
                            width: percent(80),
                            height: px(2),
                            margin: UiRect::all(px(10)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    ),
                    (
                        Name::new("Spawn Spinner Ship Button"),
                        button("Spawn Spinner Ship"),
                        observe(create_new_spaceship_spinner),
                    ),
                    (
                        Name::new("Spawn Huge Ship Button"),
                        button("Spawn Huge Ship"),
                        observe(create_new_spaceship_big),
                    ),
                    (
                        Name::new("Spawn Basic Ship Button"),
                        button("Spawn Basic Ship"),
                        observe(create_new_spaceship_basic),
                    ),
                    (
                        Name::new("Separator 4"),
                        Node {
                            width: percent(80),
                            height: px(2),
                            margin: UiRect::all(px(10)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                    ),
                    (
                        Name::new("Play Button"),
                        button("Play"),
                        observe(continue_to_simulation),
                    ),
                ],
            )],
        ));
    }

    fn setup_grab_cursor(primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
        let mut primary_cursor_options = primary_cursor_options.into_inner();
        primary_cursor_options.grab_mode = CursorGrabMode::None;
        primary_cursor_options.visible = true;
    }

    fn create_new_spaceship(
        _activate: On<Activate>,
        mut commands: Commands,
        q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
        game_assets: Res<GameAssets>,
    ) {
        for entity in &q_spaceship {
            commands.entity(entity).despawn();
        }
        let entity = commands
            .spawn((spaceship_root(SpaceshipConfig { ..default() }),))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn((hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),));
        });
    }

    fn create_new_spaceship_with_controller(
        _activate: On<Activate>,
        mut commands: Commands,
        q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
        // game_assets: Res<GameAssets>,
    ) {
        for entity in &q_spaceship {
            commands.entity(entity).despawn();
        }
        let entity = commands
            .spawn((spaceship_root(SpaceshipConfig { ..default() }),))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn((controller_section(ControllerSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                frequency: 4.0,
                damping_ratio: 4.0,
                max_torque: 100.0,
                ..default()
            }),));
        });
    }

    fn continue_to_simulation(
        _activate: On<Activate>,
        mut game_state: ResMut<NextState<super::SceneState>>,
    ) {
        game_state.set(super::SceneState::Simulation);
    }

    fn create_new_spaceship_spinner(
        _activate: On<Activate>,
        mut commands: Commands,
        q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
        game_assets: Res<GameAssets>,
    ) {
        for entity in &q_spaceship {
            commands.entity(entity).despawn();
        }

        commands.spawn((
            spaceship_root(SpaceshipConfig { ..default() }),
            children![
                (hull_section(HullSectionConfig {
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    render_mesh: Some(game_assets.hull_01.clone()),
                    ..default()
                }),),
                (hull_section(HullSectionConfig {
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    render_mesh: Some(game_assets.hull_01.clone()),
                    ..default()
                }),),
                (hull_section(HullSectionConfig {
                    transform: Transform::from_xyz(0.0, 0.0, -1.0),
                    render_mesh: Some(game_assets.hull_01.clone()),
                    ..default()
                }),),
                (
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 1.0,
                        transform: Transform::from_xyz(1.0, 0.0, 1.0)
                            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                        ..default()
                    }),
                    super::ThrusterInputKey(KeyCode::Space)
                ),
                (
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 1.0,
                        transform: Transform::from_xyz(-1.0, 0.0, -1.0)
                            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                        ..default()
                    }),
                    super::ThrusterInputKey(KeyCode::Space)
                ),
            ],
        ));
    }

    fn create_new_spaceship_big(
        _activate: On<Activate>,
        mut commands: Commands,
        q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
        game_assets: Res<GameAssets>,
    ) {
        for entity in &q_spaceship {
            commands.entity(entity).despawn();
        }
        let entity = commands
            .spawn((spaceship_root(SpaceshipConfig { ..default() }),))
            .id();

        let cube_size = 5;
        for x in -cube_size..=cube_size {
            for y in -cube_size..=cube_size {
                for z in -cube_size..=cube_size {
                    commands.entity(entity).with_children(|parent| {
                        parent.spawn((hull_section(HullSectionConfig {
                            transform: Transform::from_xyz(
                                x as f32 * 1.0,
                                y as f32 * 1.0,
                                z as f32 * 1.0,
                            ),
                            render_mesh: Some(game_assets.hull_01.clone()),
                            ..default()
                        }),));
                    });
                }
            }
        }

        let z = cube_size + 1;
        for x in -cube_size..=cube_size {
            for y in -cube_size..=cube_size {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        thruster_section(ThrusterSectionConfig {
                            magnitude: 1.0,
                            transform: Transform::from_xyz(
                                x as f32 * 1.0,
                                y as f32 * 1.0,
                                z as f32 * 1.0,
                            ),
                            ..default()
                        }),
                        super::ThrusterInputKey(KeyCode::Space),
                    ));
                });
            }
        }
    }

    fn create_new_spaceship_basic(
        _activate: On<Activate>,
        mut commands: Commands,
        q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
        game_assets: Res<GameAssets>,
    ) {
        for entity in &q_spaceship {
            commands.entity(entity).despawn();
        }

        commands.spawn((
            spaceship_root(SpaceshipConfig { ..default() }),
            children![
                (hull_section(HullSectionConfig {
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    render_mesh: Some(game_assets.hull_01.clone()),
                    ..default()
                }),),
                (hull_section(HullSectionConfig {
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    render_mesh: Some(game_assets.hull_01.clone()),
                    ..default()
                }),),
                (hull_section(HullSectionConfig {
                    transform: Transform::from_xyz(0.0, 0.0, -1.0),
                    render_mesh: Some(game_assets.hull_01.clone()),
                    ..default()
                }),),
                (
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 1.0,
                        transform: Transform::from_xyz(0.0, 0.0, 2.0),
                        ..default()
                    }),
                    super::ThrusterInputKey(KeyCode::KeyW)
                ),
                (
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 0.1,
                        transform: Transform::from_xyz(-1.0, 0.0, 1.0)
                            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                        ..default()
                    }),
                    super::ThrusterInputKey(KeyCode::KeyA)
                ),
                (
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 0.1,
                        transform: Transform::from_xyz(1.0, 0.0, 1.0)
                            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                        ..default()
                    }),
                    super::ThrusterInputKey(KeyCode::KeyD)
                ),
                (
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 0.1,
                        transform: Transform::from_xyz(-1.0, 0.0, -1.0)
                            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                        ..default()
                    }),
                    super::ThrusterInputKey(KeyCode::KeyD)
                ),
                (
                    thruster_section(ThrusterSectionConfig {
                        magnitude: 0.1,
                        transform: Transform::from_xyz(1.0, 0.0, -1.0)
                            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                        ..default()
                    }),
                    super::ThrusterInputKey(KeyCode::KeyA)
                ),
            ],
        ));
    }

    fn sections() -> impl Bundle {
        (
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                width: percent(100),
                ..default()
            },
            children![
                (
                    Name::new("Hull Section"),
                    button("Hull Section"),
                    SectionChoice::HullSection,
                ),
                (
                    Name::new("Thruster Section"),
                    button("Thruster Section"),
                    SectionChoice::ThrusterSection,
                ),
                (
                    Name::new("Turret Section"),
                    button("Turret Section"),
                    SectionChoice::TurretSection,
                ),
                (
                    Name::new("Delete Section"),
                    button("Delete Section"),
                    SectionChoice::Delete,
                ),
            ],
        )
    }

    #[derive(Component)]
    struct SectionPreviewMarker;

    fn on_click_spaceship_section(
        click: On<Pointer<Press>>,
        mut commands: Commands,
        spaceship: Single<Entity, With<SpaceshipRootMarker>>,
        q_pointer: Query<&PointerInteraction>,
        q_section: Query<&Transform, With<SectionMarker>>,
        selection: Res<SectionChoice>,
        game_assets: Res<GameAssets>,
        q_preview: Query<Entity, With<SectionPreviewMarker>>,
        keyboard: Res<ButtonInput<KeyCode>>,
    ) {
        if click.button != PointerButton::Primary {
            return;
        }

        let entity = click.entity;

        let Some(normal) = q_pointer
            .iter()
            .filter_map(|interaction| interaction.get_nearest_hit())
            .find_map(|(e, hit)| if *e == entity { hit.normal } else { None })
        else {
            return;
        };

        let Ok(transform) = q_section.get(entity) else {
            return;
        };

        let spaceship = spaceship.into_inner();
        let position = transform.translation + normal * 1.0;

        match *selection {
            SectionChoice::None => {}
            SectionChoice::HullSection => {
                commands.entity(spaceship).with_children(|parent| {
                    parent.spawn((hull_section(HullSectionConfig {
                        transform: Transform {
                            translation: position,
                            ..default()
                        },
                        render_mesh: Some(game_assets.hull_01.clone()),
                        ..default()
                    }),));
                });
            }
            SectionChoice::ThrusterSection => {
                let rotation = Quat::from_rotation_arc(Vec3::Z, normal.normalize());
                let bind = keyboard.get_pressed().next().map_or(KeyCode::Space, |k| *k);

                commands.entity(spaceship).with_children(|parent| {
                    parent.spawn((
                        thruster_section(ThrusterSectionConfig {
                            magnitude: 1.0,
                            transform: Transform {
                                translation: position,
                                rotation,
                                ..default()
                            },
                            ..default()
                        }),
                        super::ThrusterInputKey(bind),
                    ));
                });
            }
            SectionChoice::TurretSection => {
                let rotation = Quat::from_rotation_arc(Vec3::Y, normal.normalize());

                commands.entity(spaceship).with_children(|parent| {
                    parent.spawn((turret_section(TurretSectionConfig {
                        transform: Transform {
                            translation: position,
                            rotation,
                            ..default()
                        },
                        render_mesh_yaw: Some(game_assets.turret_yaw_01.clone()),
                        render_mesh_pitch: Some(game_assets.turret_pitch_01.clone()),
                        pitch_offset: Vec3::new(0.0, 0.332706, 0.303954),
                        render_mesh_barrel: Some(game_assets.turret_barrel_01.clone()),
                        barrel_offset: Vec3::new(0.0, 0.128437, -0.110729),
                        muzzle_offset: Vec3::new(0.0, 0.0, -1.2),
                        fire_rate: 100.0,
                        projectile: BulletProjectileConfig {
                            muzzle_speed: 100.0,
                            lifetime: 5.0,
                            mass: 0.1,
                            render_mesh: None,
                        },
                        ..default()
                    }),));
                });
            }
            SectionChoice::Delete => {
                commands.entity(entity).despawn();
                for preview in &q_preview {
                    commands.entity(preview).despawn();
                }
            }
        }
    }

    fn on_hover_spaceship_section(
        hover: On<Pointer<Over>>,
        mut commands: Commands,
        q_pointer: Query<&PointerInteraction>,
        q_section: Query<&GlobalTransform, With<SectionMarker>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        selection: Res<SectionChoice>,
    ) {
        let entity = hover.entity;

        let Some(normal) = q_pointer
            .iter()
            .filter_map(|interaction| interaction.get_nearest_hit())
            .find_map(|(e, hit)| if *e == entity { hit.normal } else { None })
        else {
            return;
        };

        let Ok(transform) = q_section.get(entity) else {
            return;
        };

        match *selection {
            SectionChoice::None => {}
            SectionChoice::Delete => {
                let position = transform.translation();

                commands.spawn((
                    SectionPreviewMarker,
                    Mesh3d(meshes.add(Cuboid::new(1.01, 1.01, 1.01))),
                    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
                    Transform {
                        translation: position,
                        ..default()
                    },
                ));
            }
            _ => {
                let position = transform.translation() + normal * 1.0;
                let rotation = Quat::from_rotation_arc(Vec3::Z, normal.normalize());

                commands.spawn((
                    SectionPreviewMarker,
                    Mesh3d(meshes.add(Cuboid::new(1.01, 1.01, 1.01))),
                    MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))),
                    Transform {
                        translation: position,
                        rotation,
                        ..default()
                    },
                ));
            }
        }
    }

    fn on_move_spaceship_section(
        move_: On<Pointer<Move>>,
        q_pointer: Query<&PointerInteraction>,
        q_section: Query<&GlobalTransform, With<SectionMarker>>,
        preview: Single<&mut Transform, With<SectionPreviewMarker>>,
        selection: Res<SectionChoice>,
    ) {
        if matches!(*selection, SectionChoice::Delete | SectionChoice::None) {
            return;
        }

        let entity = move_.entity;

        let Some(normal) = q_pointer
            .iter()
            .filter_map(|interaction| interaction.get_nearest_hit())
            .find_map(|(e, hit)| if *e == entity { hit.normal } else { None })
        else {
            return;
        };

        let Ok(transform) = q_section.get(entity) else {
            return;
        };

        let position = transform.translation() + normal * 1.0;
        let rotation = Quat::from_rotation_arc(Vec3::Z, normal.normalize());

        let mut preview_transform = preview.into_inner();
        preview_transform.translation = position;
        preview_transform.rotation = rotation;
    }

    fn on_out_spaceship_section(
        out: On<Pointer<Out>>,
        q_section: Query<&Transform, With<SectionMarker>>,
        mut commands: Commands,
        preview: Single<Entity, With<SectionPreviewMarker>>,
    ) {
        let Ok(_) = q_section.get(out.entity) else {
            return;
        };

        commands.entity(preview.into_inner()).despawn();
    }
}
