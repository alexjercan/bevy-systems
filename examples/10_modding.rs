use bevy::{platform::collections::HashMap, prelude::*};
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
pub struct OnUpdate;

fn custom_plugin(app: &mut App) {
    app.add_systems(Startup, setup_handler);
    app.add_systems(Update, update_system);
}

fn update_system(mut commands: Commands) {
    commands.fire::<OnUpdate>(HashMap::new());
}

#[derive(Debug, Clone)]
struct PrintEventAction {
    message: String,
}

impl EventAction for PrintEventAction {
    fn action(&self, _: &mut Commands) {
        info!("{}", self.message);
    }
}

fn setup_handler(mut commands: Commands) {
    commands.spawn((
        Name::new("OnUpdate Handler"),
        EventHandler::<OnUpdate>::new().with_action(PrintEventAction {
            message: "OnUpdate event fired!".to_string(),
        }),
    ));
}
