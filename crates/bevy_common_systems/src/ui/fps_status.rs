use bevy::prelude::*;

pub mod prelude {
    pub use super::status_fps_value_fn;
}

pub fn status_fps_value_fn() -> impl Fn(&World) -> Option<u32> + Send + Sync + 'static {
    move |world: &World| {
        let store = world.resource::<bevy::diagnostic::DiagnosticsStore>();
        store
            .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.average())
            .map(|v| v.round() as u32)
    }
}
