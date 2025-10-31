use super::handler::EventHandler;
use super::kind::EventKind;
use super::registry::RegisteredEventKind;
use bevy::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct GameEventInfo {
    pub data: Option<serde_json::Value>,
}

impl GameEventInfo {
    pub fn from_data<T: serde::Serialize>(data: T) -> Self {
        let json_value = serde_json::to_value(data).ok();
        Self { data: json_value }
    }
}

impl<T: serde::Serialize> From<T> for GameEventInfo {
    fn from(value: T) -> Self {
        GameEventInfo::from_data(value)
    }
}

#[derive(Event, Debug, Clone)]
pub struct GameEvent<E: EventKind> {
    pub(super) info: E::Info,
}

impl<E: EventKind> Default for GameEvent<E> {
    fn default() -> Self {
        Self::new()
    }
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
    E::Info: Into<GameEventInfo>,
{
    let event = event.event();
    trace!(
        "on_game_event: event {:?}, info {:?}",
        E::name(),
        event.info
    );

    for handler in &q_handler {
        if handler.filter(&event.info.clone().into()) {
            for action in &handler.actions {
                trace!("on_game_event: executing action {:?}", action.name());
                action.action(&mut commands, &event.info.clone().into());
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
