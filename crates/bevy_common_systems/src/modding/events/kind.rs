pub trait EventKind: std::fmt::Debug + Send + Sync + 'static {
    type Info: Default + std::fmt::Debug + Send + Sync + 'static;

    fn name() -> &'static str;
}
