pub mod action;
pub mod filter;
pub mod game_event;
pub mod handler;
pub mod kind;
pub mod registry;

pub mod prelude {
    pub use super::{
        action::EventAction,
        filter::EventFilter,
        game_event::{CommandsGameEventExt, GameEvent, GameEventInfo, GameEventsPlugin},
        handler::EventHandler,
        kind::EventKind,
    };
}
