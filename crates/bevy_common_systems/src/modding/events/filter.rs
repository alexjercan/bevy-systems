pub trait EventFilter: std::fmt::Debug + Send + Sync {
    type Info;

    fn filter(&self, info: &Self::Info) -> bool;

    fn name(&self) -> &'static str { std::any::type_name::<Self>() }
}
