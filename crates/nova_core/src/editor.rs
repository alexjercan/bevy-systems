//! TODO: refactor this in a nicer way once I know what I want to do with the editor, for now it is
//! used only for prototyping the spaceship editor UI

// https://github.com/bevyengine/bevy/blob/release-0.17.2/examples/ui/standard_widgets_observers.rs

use bevy::{
    picking::{hover::Hovered, pointer::PointerInteraction},
    prelude::*,
    reflect::Is,
    ui::{InteractionDisabled, Pressed},
    ui_widgets::{observe, Activate, Button},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use nova_assets::prelude::*;
use nova_gameplay::prelude::*;

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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EditorPluginSet;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SectionChoice::None);

        app.add_systems(
            OnEnter(super::GameStates::Editor),
            (reset_spaceship, setup_editor_scene, setup_grab_cursor),
        );
        app.add_systems(
            Update,
            lock_on_left_click.run_if(in_state(super::GameStates::Editor)),
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
            OnExit(super::GameStates::Editor),
            |mut selection: ResMut<SectionChoice>| {
                *selection = SectionChoice::None;
            },
        );
    }
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
        DespawnOnExit(super::GameStates::Editor),
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
        DespawnOnExit(super::GameStates::Editor),
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
        DespawnOnExit(super::GameStates::Editor),
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
    mut game_state: ResMut<NextState<super::GameStates>>,
) {
    game_state.set(super::GameStates::Simulation);
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
