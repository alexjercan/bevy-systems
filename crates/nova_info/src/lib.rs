pub const APP_VERSION: &str = env!("APP_VERSION");

pub mod prelude {
    pub use super::APP_VERSION;
}
