use super::action::EventAction;
use super::filter::EventFilter;
use super::kind::EventKind;
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Component, Debug, Clone)]
pub struct EventHandler<E: EventKind> {
    pub(super) filters: Vec<Arc<dyn EventFilter<Info = E::Info>>>,
    pub(super) actions: Vec<Arc<dyn EventAction>>,
}

impl<E: EventKind> EventHandler<E> {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn with_filter<F: EventFilter<Info = E::Info> + 'static>(mut self, f: F) -> Self {
        self.filters.push(Arc::new(f));
        self
    }

    pub fn add_filter<F: EventFilter<Info = E::Info> + 'static>(&mut self, f: F) {
        self.filters.push(Arc::new(f));
    }

    pub fn with_action<A: EventAction + 'static>(mut self, a: A) -> Self {
        self.actions.push(Arc::new(a));
        self
    }

    pub fn add_action<A: EventAction + 'static>(&mut self, a: A) {
        self.actions.push(Arc::new(a));
    }

    pub fn filter(&self, info: &E::Info) -> bool {
        self.filters.iter().all(|f| f.filter(info))
    }
}
