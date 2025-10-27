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

    // We start in the editor state
    app.add_systems(
        OnEnter(GameStates::Simulation),
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
        prelude::*,
        window::{CursorGrabMode, CursorOptions, PrimaryWindow},
    };
    use bevy_enhanced_input::prelude::*;
    use nova_protocol::prelude::*;
    use rand::prelude::*;

    pub fn simulation_plugin(app: &mut App) {
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

        app.add_systems(Update, update_text_hud);
    }

    #[derive(Component, Debug, Clone)]
    struct HealthHudMarker;

    fn setup_hud(mut commands: Commands, spaceship: Single<Entity, With<SpaceshipRootMarker>>) {
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            velocity_hud(VelocityHudConfig {
                radius: 5.0,
                target: Some(*spaceship),
            }),
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

    fn setup_scene(mut commands: Commands, game_assets: Res<GameAssets>) {
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

        // Spawn a 3D camera with a chase camera component
        commands.spawn((
            DespawnOnExit(super::SceneState::Simulation),
            Name::new("Chase Camera"),
            Camera3d::default(),
            ChaseCamera::default(),
            SpaceshipCameraControllerMarker,
            Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            SkyboxConfig {
                cubemap: game_assets.cubemap.clone(),
                brightness: 1000.0,
            },
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

    #[derive(Component, Debug, Clone)]
    struct PlayerInputMarker;

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
        mut q_input: Query<
            &mut PointRotationInput,
            (
                With<SpaceshipCameraInputMarker>,
                With<SpaceshipRotationInputActiveMarker>,
            ),
        >,
    ) {
        for mut input in &mut q_input {
            **input = fire.value;
        }
    }

    fn on_rotation_input_completed(
        _: On<Complete<CameraInputRotate>>,
        mut q_input: Query<&mut PointRotationInput, With<SpaceshipCameraInputMarker>>,
    ) {
        for mut input in &mut q_input {
            **input = Vec2::ZERO;
        }
    }

    fn on_free_mode_input_started(
        _: On<Start<FreeLookInput>>,
        mut mode: ResMut<SpaceshipCameraControlMode>,
    ) {
        *mode = SpaceshipCameraControlMode::FreeLook;
    }

    fn on_free_mode_input_completed(
        _: On<Complete<FreeLookInput>>,
        mut mode: ResMut<SpaceshipCameraControlMode>,
    ) {
        *mode = SpaceshipCameraControlMode::Normal;
    }

    fn on_combat_input_started(_: On<Start<CombatInput>>, mut mode: ResMut<SpaceshipCameraControlMode>) {
        *mode = SpaceshipCameraControlMode::Turret;
    }

    fn on_combat_input_completed(
        _: On<Complete<CombatInput>>,
        mut mode: ResMut<SpaceshipCameraControlMode>,
    ) {
        *mode = SpaceshipCameraControlMode::Normal;
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
        spaceship: Single<Entity, With<SpaceshipRootMarker>>,
        q_sections: Query<(Entity, &ChildOf), With<SectionMarker>>,
    ) {
        let spaceship = spaceship.into_inner();
        let sections = q_sections
            .iter()
            .filter(|(_, parent)| parent.0 == spaceship)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();

        commands
            .spawn((
                spaceship_root(SpaceshipConfig { ..default() }),
                PlayerSpaceshipMarker,
            ))
            .add_children(&sections);
        commands
            .entity(spaceship)
            .remove_children(&sections)
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
            .spawn((
                spaceship_root(SpaceshipConfig { ..default() }),
                PlayerSpaceshipMarker,
            ))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                base_section(BaseSectionConfig {
                    name: "Basic Hull Section".to_string(),
                    description: "A basic hull section for spaceships.".to_string(),
                    mass: 1.0,
                }),
                hull_section(HullSectionConfig {
                    render_mesh: Some(game_assets.hull_01.clone()),
                    ..default()
                }),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
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
            .spawn((
                spaceship_root(SpaceshipConfig { ..default() }),
                PlayerSpaceshipMarker,
            ))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                base_section(BaseSectionConfig {
                    name: "Basic Controller Section".to_string(),
                    description: "A basic controller section for spaceships.".to_string(),
                    mass: 1.0,
                }),
                controller_section(ControllerSectionConfig {
                    frequency: 4.0,
                    damping_ratio: 4.0,
                    max_torque: 100.0,
                    ..default()
                }),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
    }

    fn continue_to_simulation(
        _activate: On<Activate>,
        mut game_state: ResMut<NextState<super::SceneState>>,
    ) {
        game_state.set(super::SceneState::Simulation);
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
                    parent.spawn((
                        base_section(BaseSectionConfig {
                            name: "Basic Hull Section".to_string(),
                            description: "A basic hull section for spaceships.".to_string(),
                            mass: 1.0,
                        }),
                        hull_section(HullSectionConfig {
                            render_mesh: Some(game_assets.hull_01.clone()),
                            ..default()
                        }),
                        Transform {
                            translation: position,
                            ..default()
                        },
                    ));
                });
            }
            SectionChoice::ThrusterSection => {
                let rotation = Quat::from_rotation_arc(Vec3::Z, normal.normalize());
                let bind = keyboard.get_pressed().next().map_or(KeyCode::Space, |k| *k);

                commands.entity(spaceship).with_children(|parent| {
                    parent.spawn((
                        base_section(BaseSectionConfig {
                            name: "Basic Thruster Section".to_string(),
                            description: "A basic thruster section for spaceships.".to_string(),
                            mass: 1.0,
                        }),
                        thruster_section(ThrusterSectionConfig {
                            magnitude: 1.0,
                            ..default()
                        }),
                        super::ThrusterInputKey(bind),
                        Transform {
                            translation: position,
                            rotation,
                            ..default()
                        },
                    ));
                });
            }
            SectionChoice::TurretSection => {
                let rotation = Quat::from_rotation_arc(Vec3::Y, normal.normalize());

                commands.entity(spaceship).with_children(|parent| {
                    parent.spawn((
                        base_section(BaseSectionConfig {
                            name: "Basic Turret Section".to_string(),
                            description: "A basic turret section for spaceships.".to_string(),
                            mass: 1.0,
                        }),
                        turret_section(TurretSectionConfig {
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
                        }),
                        Transform {
                            translation: position,
                            rotation,
                            ..default()
                        },
                    ));
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
