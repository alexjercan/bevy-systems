use avian3d::prelude::*;
use bevy::{
    picking::{hover::Hovered, pointer::PointerInteraction},
    prelude::*,
    reflect::Is,
    ui::{InteractionDisabled, Pressed},
    ui_widgets::{observe, Activate, Button},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use rand::prelude::*;

use crate::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum ExampleStates {
    #[default]
    Loading,
    Editor,
    Scenario,
}

pub fn core_plugin(app: &mut App) {
    app.init_state::<ExampleStates>();
    app.insert_resource(SectionChoice::None);

    app.add_systems(
        OnEnter(GameStates::Playing),
        (|mut game_state: ResMut<NextState<ExampleStates>>| {
            game_state.set(ExampleStates::Editor);
        },),
    );

    app.add_systems(
        OnEnter(ExampleStates::Editor),
        (
            setup_editor_scene,
            setup_grab_cursor,
            |mut selection: ResMut<SectionChoice>| {
                *selection = SectionChoice::None;
            },
        ),
    );
    app.add_systems(
        OnEnter(ExampleStates::Scenario),
        (setup_scenario, |mut selection: ResMut<SectionChoice>| {
            *selection = SectionChoice::None;
        }),
    );

    app.add_observer(make_spaceship_player);

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

    app.add_systems(Update, lock_on_left_click);
    app.add_systems(
        Update,
        switch_scene_editor.run_if(in_state(ExampleStates::Scenario)),
    );

    app.configure_sets(
        Update,
        SpaceshipSystems::Input.run_if(in_state(ExampleStates::Scenario)),
    );
}

fn setup_scenario(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.trigger(LoadScenario(test_scenario(&game_assets)));
}

fn switch_scene_editor(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<ExampleStates>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::F1) {
        debug!("switch_scene_editor: F1 pressed, switching to Editor state.");
        state.set(ExampleStates::Editor);
        commands.trigger(UnloadScenario);
    }
}

#[derive(Component, Debug, Clone, Reflect)]
struct SpaceshipEditorMarker;

fn make_spaceship_player(
    _: On<ScenarioLoaded>,
    mut commands: Commands,
    q_spaceship: Query<Entity, With<SpaceshipEditorMarker>>,
) {
    for entity in &q_spaceship {
        commands.entity(entity).insert((
            ScenarioScopedMarker,
            PlayerSpaceshipMarker,
            Name::new("Player Spaceship".to_string()),
            EntityId::new("player_spaceship"),
            EntityTypeName::new("spaceship"),
        ));
    }
}

pub fn test_scenario(game_assets: &GameAssets) -> ScenarioConfig {
    let mut rng = rand::rng();

    let mut objects = Vec::new();
    for i in 0..20 {
        let pos = Vec3::new(
            rng.random_range(-100.0..100.0),
            rng.random_range(-20.0..20.0),
            rng.random_range(-100.0..100.0),
        );
        let radius = rng.random_range(1.0..3.0);
        let color = Color::srgb(
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
        );

        objects.push(GameObjectConfig::Asteroid(AsteroidConfig {
            id: format!("asteroid_{}", i),
            name: format!("Asteroid {}", i),
            position: pos,
            rotation: Quat::IDENTITY,
            radius,
            color,
            health: 100.0,
        }));
    }

    let spaceship = SpaceshipConfig {
        id: "other_spaceship".to_string(),
        name: "Other Spaceship".to_string(),
        position: Vec3::new(
            rng.random_range(-50.0..50.0),
            rng.random_range(-10.0..10.0),
            rng.random_range(-50.0..50.0),
        ),
        rotation: Quat::IDENTITY,
        health: 100.0,
        controller: SpaceshipController::AI(AIControllerConfig {}),
        sections: vec![
            SpaceshipSectionConfig {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Controller Section".to_string(),
                        description: "A basic controller section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Controller(ControllerSectionConfig {
                        frequency: 4.0,
                        damping_ratio: 4.0,
                        max_torque: 100.0,
                        render_mesh: None,
                    }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 1.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Hull Section".to_string(),
                        description: "A basic hull section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Hull(HullSectionConfig { render_mesh: None }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, -1.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Hull Section".to_string(),
                        description: "A basic hull section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Hull(HullSectionConfig { render_mesh: None }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 2.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Thruster Section".to_string(),
                        description: "A basic thruster section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Thruster(ThrusterSectionConfig {
                        magnitude: 1.0,
                        render_mesh: None,
                    }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, -2.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Turret Section".to_string(),
                        description: "A basic turret section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Turret(TurretSectionConfig {
                        yaw_speed: std::f32::consts::PI,
                        pitch_speed: std::f32::consts::PI,
                        min_pitch: Some(-std::f32::consts::FRAC_PI_6),
                        max_pitch: Some(std::f32::consts::FRAC_PI_2),
                        render_mesh_base: None,
                        base_offset: Vec3::new(0.0, -0.5, 0.0),
                        render_mesh_yaw: Some(game_assets.turret_yaw_01.clone()),
                        yaw_offset: Vec3::new(0.0, 0.1, 0.0),
                        render_mesh_pitch: Some(game_assets.turret_pitch_01.clone()),
                        pitch_offset: Vec3::new(0.0, 0.332706, 0.303954),
                        render_mesh_barrel: Some(game_assets.turret_barrel_01.clone()),
                        barrel_offset: Vec3::new(0.0, 0.128437, -0.110729),
                        muzzle_offset: Vec3::new(0.0, 0.0, -1.2),
                        fire_rate: 100.0,
                        muzzle_speed: 100.0,
                        projectile_lifetime: 5.0,
                        projectile_mass: 0.1,
                        projectile_render_mesh: None,
                        muzzle_effect: None,
                    }),
                },
            },
        ],
    };
    objects.push(GameObjectConfig::Spaceship(spaceship));

    let events = vec![
        ScenarioEventConfig {
            name: EventConfig::OnStart,
            filters: vec![],
            actions: vec![EventActionConfig::Objective(ObjectiveActionConfig::new(
                "destroy_spaceship",
                "Objective: Destroy the other spaceship.",
            ))],
        },
        ScenarioEventConfig {
            name: EventConfig::OnDestroyed,
            filters: vec![EventFilterConfig::Entity(EntityFilterConfig {
                id: Some("player_spaceship".to_string()),
                type_name: None,
            })],
            actions: vec![EventActionConfig::DebugMessage(DebugMessageActionConfig {
                message: "The player's spaceship was destroyed!".to_string(),
            })],
        },
        ScenarioEventConfig {
            name: EventConfig::OnDestroyed,
            filters: vec![EventFilterConfig::Entity(EntityFilterConfig {
                id: Some("other_spaceship".to_string()),
                type_name: None,
            })],
            actions: vec![
                EventActionConfig::DebugMessage(DebugMessageActionConfig {
                    message: "Objective Complete: Destroyed the other spaceship!".to_string(),
                }),
                EventActionConfig::ObjectiveComplete(ObjectiveCompleteActionConfig {
                    id: "destroy_spaceship".to_string(),
                }),
            ],
        },
    ];

    ScenarioConfig {
        id: "test_scenario".to_string(),
        name: "Test Scenario".to_string(),
        description: "A test scenario.".to_string(),
        map: MapConfig {
            cubemap: game_assets.cubemap.clone(),
            objects,
        },
        events,
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const BORDER_COLOR_INACTIVE: Color = Color::srgb(0.25, 0.25, 0.25);
const BORDER_COLOR_ACTIVE: Color = Color::srgb(0.75, 0.52, 0.99);

const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

fn setup_editor_scene(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        DespawnOnExit(ExampleStates::Editor),
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
        DespawnOnExit(ExampleStates::Editor),
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
        DespawnOnExit(ExampleStates::Editor),
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

#[derive(Resource, Default, Debug, Component, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum SectionChoice {
    #[default]
    None,
    HullSection,
    ThrusterSection,
    TurretSection,
    Delete,
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

pub fn setup_grab_cursor(primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
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
            SpaceshipRootMarker,
            SpaceshipEditorMarker,
            Name::new("Spaceship Prefab"),
            Transform::default(),
            RigidBody::Dynamic,
            Visibility::Visible,
            Health::new(1000.0),
            ExplodableEntityMarker,
        ))
        .id();

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            base_section(BaseSectionConfig {
                name: "Basic Hull Section".to_string(),
                description: "A basic hull section for spaceships.".to_string(),
                mass: 1.0,
            }),
            Transform::from_xyz(0.0, 0.0, 0.0),
            hull_section(HullSectionConfig {
                render_mesh: Some(game_assets.hull_01.clone()),
            }),
        ));
    });
}

fn create_new_spaceship_with_controller(
    _activate: On<Activate>,
    mut commands: Commands,
    q_spaceship: Query<Entity, With<SpaceshipRootMarker>>,
) {
    for entity in &q_spaceship {
        commands.entity(entity).despawn();
    }
    let entity = commands
        .spawn((
            SpaceshipRootMarker,
            SpaceshipEditorMarker,
            Name::new("Spaceship Prefab with Controller"),
            Transform::default(),
            RigidBody::Dynamic,
            Visibility::Visible,
            Health::new(1000.0),
            ExplodableEntityMarker,
        ))
        .id();

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            base_section(BaseSectionConfig {
                name: "Basic Controller Section".to_string(),
                description: "A basic controller section for spaceships.".to_string(),
                mass: 1.0,
            }),
            Transform::from_xyz(0.0, 0.0, 0.0),
            controller_section(ControllerSectionConfig {
                frequency: 4.0,
                damping_ratio: 4.0,
                max_torque: 100.0,
                ..default()
            }),
        ));
    });
}

fn continue_to_simulation(
    _activate: On<Activate>,
    mut game_state: ResMut<NextState<ExampleStates>>,
) {
    game_state.set(ExampleStates::Scenario);
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
                    SpaceshipThrusterInputKey(bind),
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
                        muzzle_speed: 100.0,
                        projectile_lifetime: 5.0,
                        projectile_mass: 0.1,
                        ..default()
                    }),
                    SpaceshipTurretInputKey(MouseButton::Left),
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
