//! TODO: Document this module

use std::{collections::VecDeque, sync::Arc};

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        CommandsGameEventExt, EventAction, EventFilter, EventHandler, EventKind, EventWorld,
        GameEvent, GameEventInfo, GameEventsPlugin,
    };
}

pub trait EventWorld: Resource + Send + Sync {
    fn world_to_state_system(world: &mut World);
    fn state_to_world_system(world: &mut World);
}

pub trait EventKind: Clone + Send + Sync + 'static {
    type Info: serde::Serialize + Default + Clone + std::fmt::Debug + Send + Sync + 'static;

    fn name() -> &'static str;
}

pub trait EventAction<W: EventWorld>: std::fmt::Debug + Send + Sync {
    fn action(&self, world: &mut W, info: &GameEventInfo);

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub trait EventFilter<W: EventWorld>: std::fmt::Debug + Send + Sync {
    fn filter(&self, world: &W, info: &GameEventInfo) -> bool;

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(Component, Clone, Reflect)]
pub struct EventHandler<W: EventWorld> {
    pub(super) name: &'static str,
    pub(super) filters: Vec<Arc<dyn EventFilter<W>>>,
    pub(super) actions: Vec<Arc<dyn EventAction<W>>>,
}

impl<W> EventHandler<W>
where
    W: EventWorld,
{
    pub fn new<E: EventKind>() -> Self {
        Self {
            name: E::name(),
            filters: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn with_filter<F: EventFilter<W> + 'static>(mut self, f: F) -> Self {
        self.filters.push(Arc::new(f));
        self
    }

    pub fn add_filter<F: EventFilter<W> + 'static>(&mut self, f: F) {
        self.filters.push(Arc::new(f));
    }

    pub fn with_action<A: EventAction<W> + 'static>(mut self, a: A) -> Self {
        self.actions.push(Arc::new(a));
        self
    }

    pub fn add_action<A: EventAction<W> + 'static>(&mut self, a: A) {
        self.actions.push(Arc::new(a));
    }

    pub fn filter(&self, world: &W, info: &GameEventInfo) -> bool {
        self.filters.iter().all(|f| f.filter(world, info))
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
pub struct GameEvent {
    pub(super) name: &'static str,
    pub(super) info: GameEventInfo,
}

impl GameEvent {
    pub fn new(name: &'static str, info: GameEventInfo) -> Self {
        Self { name, info }
    }
}

pub trait CommandsGameEventExt {
    fn fire<E: EventKind>(&mut self, info: E::Info);
}

impl<'w, 's> CommandsGameEventExt for Commands<'w, 's> {
    fn fire<E: EventKind>(&mut self, info: E::Info) {
        self.trigger(GameEvent::new(E::name(), info.into()));
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GameEventQueue<W> {
    pub events: VecDeque<GameEvent>,
    _marker: std::marker::PhantomData<W>,
}

pub struct GameEventsPlugin<W> {
    _marker: std::marker::PhantomData<W>,
}

impl<W> Default for GameEventsPlugin<W> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<W> Plugin for GameEventsPlugin<W>
where
    W: EventWorld + Default,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<GameEventQueue<W>>();
        app.add_observer(on_game_event::<W>);

        app.init_resource::<W>();
        app.add_systems(
            PostUpdate,
            (
                W::world_to_state_system,
                queue_system::<W>,
                W::state_to_world_system,
            )
                .chain()
                .run_if(not(is_queue_empty::<W>)),
        );
    }
}

fn is_queue_empty<W>(queue: Res<GameEventQueue<W>>) -> bool
where
    W: Send + Sync + 'static,
{
    queue.events.is_empty()
}

fn on_game_event<W>(event: On<GameEvent>, mut queue: ResMut<GameEventQueue<W>>)
where
    W: Send + Sync + 'static,
{
    let event = event.event();
    trace!(
        "on_game_event: event {:?}, info {:?}",
        event.name,
        event.info
    );

    queue.events.push_back(event.clone());
}

fn queue_system<W: EventWorld>(
    mut queue: ResMut<GameEventQueue<W>>,
    mut world: ResMut<W>,
    q_handler: Query<&EventHandler<W>>,
) {
    while let Some(event) = queue.events.pop_front() {
        trace!(
            "queue_system: processing event {:?}, info {:?}",
            event.name,
            event.info
        );

        for handler in &q_handler {
            // TODO: Optimize by indexing handlers by event name
            if handler.name == event.name && handler.filter(&world, &event.info) {
                trace!("queue_system: handler {:?} passed filters", handler.name);

                for action in &handler.actions {
                    trace!("queue_system: executing action {:?}", action.name());
                    action.action(&mut world, &event.info);
                }
            }
        }
    }
}
