use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "05_turret")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to spawn a basic turret in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameStates::Playing), setup_scenario);
}

fn setup_scenario(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.trigger(LoadScenario(test_scenario(&game_assets)));
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
        let texture = game_assets.asteroid_texture.clone();

        objects.push(GameObjectConfig::Asteroid(AsteroidConfig {
            id: format!("asteroid_{}", i),
            name: format!("Asteroid {}", i),
            position: pos,
            rotation: Quat::IDENTITY,
            radius,
            texture,
            health: 100.0,
        }));
    }

    let spaceship = SpaceshipConfig {
        id: "player_spaceship".to_string(),
        name: "Player Spaceship".to_string(),
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        controller: SpaceshipController::Player(PlayerControllerConfig {}),
        health: 500.0,
        sections: vec![
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
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
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 1.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Turret Section".to_string(),
                        description: "A basic turret section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Turret(TurretSectionConfig::default()),
                },
            },
        ],
    };
    objects.push(GameObjectConfig::Spaceship(spaceship));

    ScenarioConfig {
        id: "test_scenario".to_string(),
        name: "Test Scenario".to_string(),
        description: "A test scenario.".to_string(),
        map: MapConfig {
            cubemap: game_assets.cubemap.clone(),
            objects,
        },
        events: vec![],
    }
}
