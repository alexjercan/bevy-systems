use bevy::prelude::*;
use clap::Parser;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "10_modding")]
#[command(version = "1.0.0")]
#[command(about = "A simple example showing how to create a basic scene in nova_protocol with custom events", long_about = None)]
struct Cli;

fn main() {
    let _ = Cli::parse();
    let mut app = AppBuilder::new().with_game_plugins(custom_plugin).build();

    app.run();
}

#[derive(Debug, Clone, EventKind)]
#[event_name("onupdate")]
#[event_info(OnUpdateInfo)]
pub struct OnUpdate;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OnUpdateInfo {
    value: f32,
}

fn custom_plugin(app: &mut App) {
    app.add_systems(Startup, setup_handler);
    app.add_systems(Update, update_system);
}

fn update_system(mut commands: Commands) {
    commands.fire::<OnUpdate>(OnUpdateInfo {
        value: rand::random(),
    });
}

#[derive(Debug, Clone)]
struct PrintEventAction {
    message: String,
}

impl EventAction for PrintEventAction {
    fn action(&self, _: &mut Commands, _: &GameEventInfo) {
        info!("{}", self.message);
    }
}

#[derive(Debug, Clone)]
struct MyCustomFilter {
    min_value: f32,
}

impl EventFilter for MyCustomFilter {
    fn filter(&self, info: &GameEventInfo) -> bool {
        let Some(data) = &info.data else {
            return false;
        };

        let Some(value) = data.get("value").and_then(|v| v.as_f64()) else {
            return false;
        };

        return (value as f32) >= self.min_value;
    }
}

fn setup_handler(mut commands: Commands) {
    commands.spawn((
        Name::new("OnUpdate Handler"),
        EventHandler::<OnUpdate>::new()
            .with_filter(MyCustomFilter { min_value: 0.1 })
            .with_action(PrintEventAction {
                message: "OnUpdate event fired with value >= 0.1".to_string(),
            }),
    ));
}
