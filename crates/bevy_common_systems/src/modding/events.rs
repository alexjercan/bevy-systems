//! A generic, extensible event system for Bevy games.
//!
//! This module provides traits and components to define, filter, and handle game events
//! in a flexible way. Events are queued and processed in a world-specific context,
//! allowing complex systems to react to game state changes.

use std::{collections::VecDeque, sync::Arc};

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        CommandsGameEventExt, EventAction, EventFilter, EventHandler, EventKind, EventWorld,
        GameEvent, GameEventInfo, GameEventsPlugin,
    };
}

/// A trait representing a game world that can synchronize its state to and from systems.
///
/// Implement this trait for your game state resource to integrate with `GameEventsPlugin`.
pub trait EventWorld: Resource + Send + Sync {
    /// System to update the world from a saved or external state.
    fn world_to_state_system(world: &mut World);

    /// System to update the state back to the world after processing events.
    fn state_to_world_system(world: &mut World);
}

/// A trait representing a kind of game event.
///
/// Each event kind defines its data type (`Info`) and a unique name.
pub trait EventKind: Clone + Send + Sync + 'static {
    /// The type of event data associated with this event kind.
    type Info: serde::Serialize + Default + Clone + std::fmt::Debug + Send + Sync + 'static;

    /// Returns a unique name for this event type.
    fn name() -> &'static str;
}

/// A trait representing an action to perform in response to an event.
pub trait EventAction<W: EventWorld>: Send + Sync {
    /// Execute the action on the given world using the event info.
    fn action(&self, world: &mut W, info: &GameEventInfo);

    /// Returns the name of the action (defaults to the Rust type name).
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A trait representing a filter that determines if an event should trigger an action.
pub trait EventFilter<W: EventWorld>: Send + Sync {
    /// Returns true if the event passes the filter.
    fn filter(&self, world: &W, info: &GameEventInfo) -> bool;

    /// Returns the name of the filter (defaults to the Rust type name).
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Component that handles game events by applying filters and executing actions.
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
    /// Create a new handler for a given event kind.
    pub fn new<E: EventKind>() -> Self {
        Self {
            name: E::name(),
            filters: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Add a filter to the handler (builder-style).
    pub fn with_filter<F: EventFilter<W> + 'static>(mut self, f: F) -> Self {
        self.filters.push(Arc::new(f));
        self
    }

    /// Add a filter to the handler.
    pub fn add_filter<F: EventFilter<W> + 'static>(&mut self, f: F) {
        self.filters.push(Arc::new(f));
    }

    /// Add an action to the handler (builder-style).
    pub fn with_action<A: EventAction<W> + 'static>(mut self, a: A) -> Self {
        self.actions.push(Arc::new(a));
        self
    }

    /// Add an action to the handler.
    pub fn add_action<A: EventAction<W> + 'static>(&mut self, a: A) {
        self.actions.push(Arc::new(a));
    }

    /// Checks if the event passes all filters.
    pub fn filter(&self, world: &W, info: &GameEventInfo) -> bool {
        self.filters.iter().all(|f| f.filter(world, info))
    }
}

/// Event data wrapper.
#[derive(Debug, Clone, Default)]
pub struct GameEventInfo {
    /// Optional serialized data for the event.
    pub data: Option<serde_json::Value>,
}

impl GameEventInfo {
    /// Create an event info from serializable data.
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

/// Represents a fired game event in the Bevy event system.
#[derive(Event, Debug, Clone)]
pub struct GameEvent {
    pub(super) name: &'static str,
    pub(super) info: GameEventInfo,
}

impl GameEvent {
    /// Create a new game event with the given name and info.
    pub fn new(name: &'static str, info: GameEventInfo) -> Self {
        Self { name, info }
    }
}

/// Extension trait for `Commands` to fire game events easily.
pub trait CommandsGameEventExt {
    fn fire<E: EventKind>(&mut self, info: E::Info);
}

impl<'w, 's> CommandsGameEventExt for Commands<'w, 's> {
    fn fire<E: EventKind>(&mut self, info: E::Info) {
        self.trigger(GameEvent::new(E::name(), info.into()));
    }
}

/// Resource holding a queue of pending game events for a specific world type.
#[derive(Resource, Debug, Clone, Default)]
pub struct GameEventQueue<W> {
    pub events: VecDeque<GameEvent>,
    _marker: std::marker::PhantomData<W>,
}

/// Plugin that processes game events for a specific world type.
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
                .run_if(not(is_queue_empty::<W>).or(resource_changed::<W>)),
        );
    }
}

/// Returns true if the event queue is empty.
fn is_queue_empty<W>(queue: Res<GameEventQueue<W>>) -> bool
where
    W: Send + Sync + 'static,
{
    queue.events.is_empty()
}

/// Observer that pushes fired events into the queue.
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

/// Processes the event queue by applying handlers and executing actions.
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
