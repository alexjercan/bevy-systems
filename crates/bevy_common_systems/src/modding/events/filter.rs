use super::game_event::GameEventInfo;

pub trait EventFilter: std::fmt::Debug + Send + Sync {
    fn filter(&self, info: &GameEventInfo) -> bool;

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
