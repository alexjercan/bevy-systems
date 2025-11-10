use bevy::prelude::*;
use clap::Parser;
use nova_core::nova_gameplay::hud::health::HealthHudTargetEntity;
use nova_protocol::prelude::*;
use rand::prelude::*;

#[derive(Parser)]
#[command(name = "07a_health")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to manage health in nova_protocol", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

fn custom_plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameStates::Playing),
        (setup_scenario, setup_hud_health),
    );

    app.add_observer(on_click_damage_health);
    app.add_observer(on_hover_set_health_target);
}

fn setup_scenario(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.trigger(LoadScenario(test_scenario(&game_assets)));
}

fn setup_hud_health(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameStates::Playing),
        health_hud(HealthHudConfig { target: None }),
    ));
}

fn on_click_damage_health(
    click: On<Pointer<Press>>,
    mut commands: Commands,
    q_health: Query<&Health>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;
    println!("Clicked on entity: {:?}", entity);

    if let Ok(health) = q_health.get(entity) {
        println!("Entity has health: {:?}", health);

        commands.trigger(HealthApplyDamage {
            target: click.entity,
            source: None,
            amount: 10.0,
        });
    }
}

fn on_hover_set_health_target(
    hover: On<Pointer<Over>>,
    q_health: Query<&Health>,
    q_health_hud: Single<&mut HealthHudTargetEntity, With<HealthHudMarker>>,
) {
    let entity = hover.entity;

    let Ok(_) = q_health.get(entity) else {
        return;
    };

    let mut health_hud_target = q_health_hud.into_inner();
    **health_hud_target = Some(entity);
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
