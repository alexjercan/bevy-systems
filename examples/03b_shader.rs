use avian3d::prelude::*;
use bevy::{
    input_focus::InputDispatchPlugin,
    pbr::ExtendedMaterial,
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{observe, Slider, SliderRange, SliderThumb, SliderValue, ValueChange},
};
use clap::Parser;
use nova_protocol::{
    nova_gameplay::sections::thruster_section::ThrusterExhaustMaterial, prelude::*,
};
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "03b_shader")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to create the thruster shader in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = new_gui_app();

    app.insert_resource(DemoWidgetStates { slider_value: 0.0 });

    app.add_systems(
        OnEnter(GameStates::Playing),
        (
            setup_cone_shader,
            setup_camera,
            setup_simple_scene,
            setup_ui_slider,
        ),
    );

    app.add_systems(Update, thruster_shader_update_system);

    app.add_observer(slider_on_interaction::<Insert, Hovered>)
        .add_observer(slider_on_change_value::<SliderValue>)
        .add_observer(slider_on_change_value::<SliderRange>);

    app.add_systems(Update, update_widget_values);

    app.run();
}

fn thruster_shader_update_system(
    value: Res<DemoWidgetStates>,
    material: Single<&MeshMaterial3d<ExtendedMaterial<StandardMaterial, ThrusterExhaustMaterial>>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ThrusterExhaustMaterial>>>,
) {
    let material = material.into_inner();

    let Some(material) = materials.get_mut(&**material) else {
        warn!("ThrusterSectionExhaustShaderMarker's material not found");
        return;
    };

    material.extension.thruster_input = value.slider_value / 100.0;
}

fn setup_cone_shader(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut exhaust_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, ThrusterExhaustMaterial>>,
    >,
) {
    let cone_mesh = meshes.add(Cone::new(0.4, 0.1));

    commands.spawn((
        Name::new("Thruster Exhaust - without shader"),
        Transform::from_xyz(-0.0, 0.0, 0.0),
        Mesh3d(cone_mesh.clone()),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.5, 0.0, 1.0),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            ..default()
        })),
    ));

    commands.spawn((
        Name::new("Thruster Exhaust - emissive"),
        Transform::from_xyz(5.0, 0.0, 0.0),
        Mesh3d(cone_mesh.clone()),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
            perceptual_roughness: 1.0,
            metallic: 0.0,
            emissive: LinearRgba::rgb(10.0, 5.0, 0.0),
            ..default()
        })),
    ));

    commands.spawn((
        Name::new("Thruster Exhaust - with shader"),
        Transform::from_xyz(10.0, 0.0, 0.0),
        Mesh3d(cone_mesh),
        MeshMaterial3d(
            exhaust_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    perceptual_roughness: 1.0,
                    metallic: 0.0,
                    emissive: LinearRgba::rgb(10.0, 5.0, 0.0),
                    ..default()
                },
                extension: ThrusterExhaustMaterial::default()
                    .with_exhaust_height(1.0)
                    .with_exhaust_radius(0.4),
            }),
        ),
    ));
}

const SLIDER_TRACK: Color = Color::srgb(0.05, 0.05, 0.05);
const SLIDER_THUMB: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component, Default)]
struct DemoSlider;

#[derive(Component, Default)]
struct DemoSliderThumb;

#[derive(Resource)]
struct DemoWidgetStates {
    slider_value: f32,
}

fn setup_ui_slider(mut commands: Commands) {
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![(
            slider(0.0, 100.0, 0.0),
            observe(
                |value_change: On<ValueChange<f32>>,
                 mut widget_states: ResMut<DemoWidgetStates>| {
                    widget_states.slider_value = value_change.value;
                },
            )
        )],
    ));
}

/// Create a demo slider
fn slider(min: f32, max: f32, value: f32) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Center,
            column_gap: px(4),
            height: px(12),
            width: percent(30),
            ..default()
        },
        Name::new("Slider"),
        Hovered::default(),
        DemoSlider,
        Slider::default(),
        SliderValue(value),
        SliderRange::new(min, max),
        // TabIndex(0),
        Children::spawn((
            // Slider background rail
            Spawn((
                Node {
                    height: px(6),
                    ..default()
                },
                BackgroundColor(SLIDER_TRACK), // Border color for the checkbox
                BorderRadius::all(px(3)),
            )),
            // Invisible track to allow absolute placement of thumb entity. This is narrower than
            // the actual slider, which allows us to position the thumb entity using simple
            // percentages, without having to measure the actual width of the slider thumb.
            Spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: px(0),
                    // Track is short by 12px to accommodate the thumb.
                    right: px(12),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                children![(
                    // Thumb
                    DemoSliderThumb,
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: px(12),
                        height: px(12),
                        position_type: PositionType::Absolute,
                        left: percent(0), // This will be updated by the slider's value
                        ..default()
                    },
                    BorderRadius::MAX,
                    BackgroundColor(SLIDER_THUMB),
                )],
            )),
        )),
    )
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

fn slider_on_interaction<E: EntityEvent, C: Component>(
    event: On<E, C>,
    sliders: Query<(Entity, &Hovered), With<DemoSlider>>,
    children: Query<&Children>,
    mut thumbs: Query<(&mut BackgroundColor, Has<DemoSliderThumb>), Without<DemoSlider>>,
) {
    if let Ok((slider_ent, hovered)) = sliders.get(event.event_target()) {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_bg, is_thumb)) = thumbs.get_mut(child) {
                if is_thumb {
                    thumb_bg.0 = if hovered.0 {
                        SLIDER_THUMB.lighter(0.3)
                    } else {
                        SLIDER_THUMB
                    }
                }
            }
        }
    }
}

fn slider_on_change_value<C: Component>(
    insert: On<Insert, C>,
    sliders: Query<(Entity, &SliderValue, &SliderRange), With<DemoSlider>>,
    children: Query<&Children>,
    mut thumbs: Query<(&mut Node, Has<DemoSliderThumb>), Without<DemoSlider>>,
) {
    if let Ok((slider_ent, value, range)) = sliders.get(insert.entity) {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child) {
                if is_thumb {
                    thumb_node.left = percent(range.thumb_position(value.0) * 100.0);
                }
            }
        }
    }
}

fn update_widget_values(
    res: Res<DemoWidgetStates>,
    mut sliders: Query<Entity, With<DemoSlider>>,
    mut commands: Commands,
) {
    if res.is_changed() {
        for slider_ent in sliders.iter_mut() {
            commands
                .entity(slider_ent)
                .insert(SliderValue(res.slider_value));
        }
    }
}
