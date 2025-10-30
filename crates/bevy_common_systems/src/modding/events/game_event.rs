use bevy::prelude::*;
use super::kind::EventKind;
use super::handler::EventHandler;
use super::registry::RegisteredEventKind;

#[derive(Event, Debug, Clone)]
pub struct GameEvent<E: EventKind> {
    pub(super) info: E::Info,
}

impl<E: EventKind> GameEvent<E> {
    pub fn new() -> Self {
        Self {
            info: E::Info::default(),
        }
    }

    pub fn with_info(info: E::Info) -> Self {
        Self { info }
    }
}

pub trait CommandsGameEventExt {
    fn fire<E: EventKind>(&mut self, info: E::Info);
}

impl<'w, 's> CommandsGameEventExt for Commands<'w, 's> {
    fn fire<E: EventKind>(&mut self, info: E::Info) {
        self.trigger(GameEvent::<E>::with_info(info));
    }
}

pub fn on_game_event<E>(
    event: On<GameEvent<E>>,
    mut commands: Commands,
    q_handler: Query<&EventHandler<E>>,
) where
    E: EventKind,
{
    let event = event.event();
    debug!("on_game_event: event {:?}, info {:?}", E::name(), event.info);

    for handler in &q_handler {
        if handler.filter(&event.info) {
            for action in &handler.actions {
                debug!("on_game_event: executing action {:?}", action.name());
                action.action(&mut commands);
            }
        }
    }
}

pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        for event_kind in inventory::iter::<RegisteredEventKind> {
            debug!("GameEventsPlugin: register {}", event_kind.name);
            (event_kind.register_fn)(app);
        }
    }
}
