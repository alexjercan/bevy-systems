use std::sync::Arc;

use bevy::prelude::*;
pub use inventory;

pub mod prelude {
    pub use super::{
        CommandsGameEventExt, EventAction, EventFilter, EventHandler, EventKind, GameEvent,
        GameEventInfo, GameEventsPlugin,
    };
}

pub struct RegisteredEventKind {
    pub name: &'static str,
    pub register_fn: fn(&mut bevy::prelude::App),
}

inventory::collect!(RegisteredEventKind);

pub trait EventKind: Clone + std::fmt::Debug + Send + Sync + 'static {
    type Info: Default + Clone + std::fmt::Debug + Send + Sync + 'static;

    fn name() -> &'static str;
}

pub trait EventAction: std::fmt::Debug + Send + Sync {
    fn action(&self, commands: &mut Commands, info: &GameEventInfo);

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub trait EventFilter: std::fmt::Debug + Send + Sync {
    fn filter(&self, info: &GameEventInfo) -> bool;

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(Component, Debug, Clone)]
pub struct EventHandler<E: EventKind> {
    pub(super) filters: Vec<Arc<dyn EventFilter>>,
    pub(super) actions: Vec<Arc<dyn EventAction>>,
    _marker: std::marker::PhantomData<E>,
}

impl<E: EventKind> Default for EventHandler<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EventKind> EventHandler<E> {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            actions: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_filter<F: EventFilter + 'static>(mut self, f: F) -> Self {
        self.filters.push(Arc::new(f));
        self
    }

    pub fn add_filter<F: EventFilter + 'static>(&mut self, f: F) {
        self.filters.push(Arc::new(f));
    }

    pub fn with_action<A: EventAction + 'static>(mut self, a: A) -> Self {
        self.actions.push(Arc::new(a));
        self
    }

    pub fn add_action<A: EventAction + 'static>(&mut self, a: A) {
        self.actions.push(Arc::new(a));
    }

    pub fn filter(&self, info: &GameEventInfo) -> bool {
        self.filters.iter().all(|f| f.filter(info))
    }
}

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

pub trait CommandsGameEventExt {
    fn fire<E: EventKind>(&mut self, info: E::Info);
}

impl<'w, 's> CommandsGameEventExt for Commands<'w, 's> {
    fn fire<E: EventKind>(&mut self, info: E::Info) {
        self.trigger(GameEvent::<E>::with_info(info));
    }
}
