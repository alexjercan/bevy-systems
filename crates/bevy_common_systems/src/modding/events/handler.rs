use std::sync::Arc;

use bevy::prelude::*;

use super::{action::EventAction, filter::EventFilter, game_event::GameEventInfo, kind::EventKind};

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
