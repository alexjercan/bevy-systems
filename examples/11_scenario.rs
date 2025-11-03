use avian3d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use clap::Parser;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "11_scenario")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to create a basic scenario in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameAssetsStates::Loaded), setup_scenario);
    app.add_systems(OnEnter(GameStates::Simulation), setup_camera);
}

fn setup_camera(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        DespawnOnExit(GameStates::Simulation),
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

    commands.spawn((
        DespawnOnExit(GameStates::Simulation),
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
}

fn setup_scenario(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scenarios: Res<GameScenarios>,
) {
    // Here we pretend that we get the scenario from the assets
    let scenario = scenarios.first().expect("No scenario found");
    info!("Setting up scenario: {}", scenario.name);

    let mut mapping = HashMap::new();
    for object in scenario.map.objects.iter() {
        match object {
            GameObjectConfig::Asteroid(asteroid_config) => {
                let id = commands
                    .spawn((
                        DespawnOnExit(GameStates::Simulation),
                        Name::new(asteroid_config.name.clone()),
                        EntityId::new(asteroid_config.id.clone()),
                        EntityTypeName::new("asteroid"),
                        Transform::from_translation(asteroid_config.position)
                            .with_rotation(asteroid_config.rotation),
                        Mesh3d(meshes.add(Sphere::new(asteroid_config.radius))),
                        MeshMaterial3d(materials.add(asteroid_config.color)),
                        Collider::sphere(asteroid_config.radius),
                        RigidBody::Dynamic,
                        Health::new(100.0),
                        ExplodableEntityMarker,
                    ))
                    .id();

                mapping.insert(asteroid_config.id.clone(), id);
            }
            GameObjectConfig::Spaceship(spaceship_config) => {
                let id = commands
                    .spawn((
                        DespawnOnExit(GameStates::Simulation),
                        PlayerSpaceshipMarker,
                        Name::new(spaceship_config.name.clone()),
                        EntityId::new(spaceship_config.id.clone()),
                        EntityTypeName::new("spaceship"),
                        Transform::from_translation(spaceship_config.position)
                            .with_rotation(spaceship_config.rotation),
                        RigidBody::Dynamic,
                        Visibility::Visible,
                        Health::new(100.0),
                        ExplodableEntityMarker,
                    ))
                    .with_children(|parent| {
                        for section in spaceship_config.sections.iter() {
                            let mut section_entity = parent.spawn((
                                base_section(section.config.base.clone()),
                                Transform::from_translation(section.position)
                                    .with_rotation(section.rotation),
                            ));

                            match &section.config.kind {
                                SectionKind::Hull(hull_config) => {
                                    section_entity.insert(hull_section(hull_config.clone()));
                                }
                                SectionKind::Controller(controller_config) => {
                                    section_entity
                                        .insert(controller_section(controller_config.clone()));
                                }
                                SectionKind::Thruster(thruster_config) => {
                                    section_entity
                                        .insert(thruster_section(thruster_config.clone()))
                                        .insert(SpaceshipThrusterInputKey(KeyCode::Space));
                                }
                                SectionKind::Turret(turret_config) => {
                                    section_entity.insert(turret_section(turret_config.clone()));
                                }
                            }
                        }
                    })
                    .id();

                mapping.insert(spaceship_config.id.clone(), id);
            }
        }
    }

    for event in scenario.events.iter() {
        let mut event_handler = EventHandler::<NovaEventWorld>::new::<OnDestroyedEvent>();
        for filter in event.filters.iter() {
            event_handler.add_filter(filter.clone());
        }
        for action in event.actions.iter() {
            event_handler.add_action(action.clone());
        }
        commands.spawn(event_handler);
    }
}
