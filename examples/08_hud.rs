use bevy::{platform::collections::HashMap, prelude::*};
use clap::Parser;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "08_hud")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how the hud will work in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameStates::Playing), setup_scenario);
}

fn setup_scenario(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    sections: Res<GameSections>,
) {
    commands.trigger(LoadScenario(test_scenario(&game_assets, sections)));
}

pub fn test_scenario(game_assets: &GameAssets, sections: Res<GameSections>) -> ScenarioConfig {
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

        objects.push(ScenarioObjectConfig {
            base: BaseScenarioObjectConfig {
                id: format!("asteroid_{}", i),
                name: format!("Asteroid {}", i),
                position: pos,
                rotation: Quat::IDENTITY,
                health: 100.0,
            },
            kind: ScenarioObjectKind::Asteroid(AsteroidConfig { radius, texture }),
        });
    }

    let spaceship = SpaceshipConfig {
        controller: SpaceshipController::Player(PlayerControllerConfig {
            input_mapping: HashMap::from([("thruster".to_string(), KeyCode::Space)]),
        }),
        sections: vec![SpaceshipSectionConfig {
            id: "thruster".to_string(),
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            config: sections
                .get_section("basic_thruster_section")
                .unwrap()
                .clone(),
        }],
    };
    objects.push(ScenarioObjectConfig {
        base: BaseScenarioObjectConfig {
            id: "player_spaceship".to_string(),
            name: "Player Spaceship".to_string(),
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            health: 500.0,
        },
        kind: ScenarioObjectKind::Spaceship(spaceship),
    });

    let events = vec![ScenarioEventConfig {
        name: EventConfig::OnStart,
        filters: vec![],
        actions: objects
            .into_iter()
            .map(EventActionConfig::SpawnScenarioObject)
            .collect::<_>(),
    }];

    ScenarioConfig {
        id: "test_scenario".to_string(),
        name: "Test Scenario".to_string(),
        description: "A test scenario.".to_string(),
        cubemap: game_assets.cubemap.clone(),
        events,
    }
}
