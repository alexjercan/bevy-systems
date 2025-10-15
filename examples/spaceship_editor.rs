mod helpers;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_ui_widgets::UiWidgetsPlugins;
use clap::Parser;
use helpers::*;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "spaceship_editor")]
#[command(version = "0.1")]
#[command(about = "Simple spaceship editor scene where you can build custom ships", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();
    app.add_plugins(EnhancedInputPlugin);

    // Helper plugins
    app.add_plugins(GameAssetsPlugin);
    app.add_plugins(WASDCameraControllerPlugin);
    #[cfg(feature = "debug")]
    app.add_plugins(DebugGizmosPlugin);

    app.add_plugins(UiWidgetsPlugins);

    // We need to enable the physics plugins to have access to RigidBody and other components.
    // We will also disable gravity for this example, since we are in space, duh.
    app.add_plugins(PhysicsPlugins::default().set(PhysicsInterpolationPlugin::interpolate_all()));
    app.insert_resource(Gravity::ZERO);
    #[cfg(feature = "debug")]
    app.add_plugins(PhysicsDebugPlugin::default());

    // Render Plugins
    app.add_plugins(SkyboxPlugin);
    app.add_plugins(PostProcessingDefaultPlugin);

    // Add sections plugins
    app.add_plugins(SpaceshipPlugin { render: true });

    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_scene, setup_simple_scene),
    );

    app.init_state::<SceneState>();

    app.add_plugins(editor::editor_plugin);
    app.add_systems(
        Update,
        switch_scene_editor.run_if(in_state(SceneState::Simulation)),
    );
    app.add_systems(
        OnEnter(GameStates::Playing),
        |mut state: ResMut<NextState<SceneState>>| {
            state.set(SceneState::Editor);
        },
    );

    app.run();
}

fn setup_scene(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Name::new("WASD Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        SkyboxConfig {
            cubemap: game_assets.cubemap.clone(),
            brightness: 1000.0,
        },
    ));
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

mod editor {
    // https://github.com/bevyengine/bevy/blob/release-0.17.2/examples/ui/standard_widgets_observers.rs
    // TODO:
    // - on mouse click we find the position in world of the click
    // - if we clicked on a spaceship section we spawn what we have selected from the menu
    // - we will need to rotate the part appropriately based on the normal of the surface we
    // clicked on

    use crate::helpers::GameAssets;

    use super::SceneState;
    use bevy::{
        picking::hover::Hovered,
        prelude::*,
        reflect::Is,
        ui::{InteractionDisabled, Pressed},
    };
    use bevy_ui_widgets::{observe, Activate, Button};
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
    }

    pub fn editor_plugin(app: &mut App) {
        app.insert_resource(SectionChoice::None);

        app.add_systems(OnEnter(SceneState::Editor), editor_menu_setup);

        app.add_observer(button_on_interaction::<Add, Pressed>)
            .add_observer(button_on_interaction::<Remove, Pressed>)
            .add_observer(button_on_interaction::<Add, InteractionDisabled>)
            .add_observer(button_on_interaction::<Remove, InteractionDisabled>)
            .add_observer(button_on_interaction::<Insert, Hovered>);

        app.add_observer(on_add_selected)
            .add_observer(on_remove_selected);
        app.add_observer(button_on_setting::<SectionChoice>);
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
                commands.entity(previous.into_inner()).remove::<SelectedOption>();
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

    fn editor_menu_setup(mut commands: Commands) {
        commands.spawn((
            DespawnOnExit(SceneState::Editor),
            Name::new("Editor Main Menu"),
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
                        Name::new("Create New Spaceship Button"),
                        button("Create New Spaceship"),
                        observe(create_new_spaceship),
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
                    // Let's add some buttons for different sections of the spaceship
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

    fn create_new_spaceship(
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
            children![(hull_section(HullSectionConfig {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                render_mesh: Some(game_assets.hull_01.clone()),
                ..default()
            }),),],
        ));
    }

    fn continue_to_simulation(
        _activate: On<Activate>,
        mut game_state: ResMut<NextState<SceneState>>,
    ) {
        println!("Switching to simulation");
        game_state.set(SceneState::Simulation);
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
}
