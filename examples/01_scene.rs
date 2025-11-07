use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "01_scene")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to create a basic scene in nova_protocol", long_about = None)]
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
        let radius = rng.random_range(2.0..6.0);
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
