pub use inventory;

pub struct RegisteredEventKind {
    pub name: &'static str,
    pub register_fn: fn(&mut bevy::prelude::App),
}

inventory::collect!(RegisteredEventKind);
