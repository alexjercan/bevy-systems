pub trait EventKind: Clone + std::fmt::Debug + Send + Sync + 'static {
    type Info: Default + Clone + std::fmt::Debug + Send + Sync + 'static;

    fn name() -> &'static str;
}
