//! The Status plugin provides a simple status bar UI for displaying different metrics.
//! The idea is it should be easy to add what metrics to display via generic components
//! Then you update the components and the ststaus bar updates automatically.

use std::{any::Any, fmt::Display, sync::Arc};

use bevy::{platform::collections::HashMap, prelude::*};

pub mod prelude {
    pub use super::status_bar;
    pub use super::status_bar_item;
    pub use super::status_fps_color_fn;
    pub use super::status_fps_value_fn;
    pub use super::status_version_color_fn;
    pub use super::status_version_value_fn;
    pub use super::StatusBarItemConfig;
    pub use super::StatusBarItemMarker;
    pub use super::StatusBarPlugin;
    pub use super::StatusBarPluginSystems;
    pub use super::StatusBarRootConfig;
    pub use super::StatusBarRootMarker;
    pub use super::StatusValue;
}

/// The StatusBarRootMarker component is a marker component that indicates the root node of the status
/// bar UI.
#[derive(Component, Clone, Debug, Reflect)]
pub struct StatusBarRootMarker;

#[derive(Clone, Debug, Default)]
pub struct StatusBarRootConfig {}

/// --- Status bar in top-right for FPS, latency, etc ---
pub fn status_bar(_config: StatusBarRootConfig) -> impl Bundle {
    (
        Name::new("StatusBarUIRoot"),
        StatusBarRootMarker,
        Node {
            width: Val::Auto,
            height: Val::Auto,
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
    )
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct StatusBarItemMarker;

pub trait StatusValue: Any + Display + Send + Sync + 'static {}
impl<T> StatusValue for T where T: Any + Display + Send + Sync + 'static {}

/// The StatusBarItemConfig component defines a single item in the status bar.
#[derive(Debug, Clone, Default)]
pub struct StatusBarItemConfig<F, G>
where
    F: Fn(&World) -> Option<Arc<dyn StatusValue>> + Send + Sync + 'static,
    G: Fn(Box<&dyn Any>) -> Option<Color> + Send + Sync + 'static,
{
    pub icon: Option<Handle<Image>>,
    pub value_fn: F,
    pub color_fn: G,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct StatusBarItemIcon(pub Handle<Image>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct StatusBarItemPrefix(pub String);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct StatusBarItemSuffix(pub String);

#[derive(Component, Clone, Deref, DerefMut)]
pub struct StatusBarItemValueFnBoxed(
    pub Arc<dyn Fn(&World) -> Option<Arc<dyn StatusValue>> + Send + Sync>,
);

#[derive(Component, Clone, Deref, DerefMut)]
pub struct StatusBarItemColorFnBoxed(pub Arc<dyn Fn(Box<&dyn Any>) -> Option<Color> + Send + Sync>);

pub fn status_bar_item<F, G>(config: StatusBarItemConfig<F, G>) -> impl Bundle
where
    F: Fn(&World) -> Option<Arc<dyn StatusValue>> + Send + Sync + 'static,
    G: Fn(Box<&dyn Any>) -> Option<Color> + Send + Sync + 'static,
{
    (
        Name::new("StatusBarItem"),
        StatusBarItemMarker,
        StatusBarItemIcon(config.icon.unwrap_or_default()),
        StatusBarItemPrefix(config.prefix),
        StatusBarItemSuffix(config.suffix),
        StatusBarItemValueFnBoxed(Arc::new(config.value_fn)),
        StatusBarItemColorFnBoxed(Arc::new(config.color_fn)),
    )
}

#[derive(Component, Clone, Deref, DerefMut)]
pub struct StatusBarItemValue(pub Option<Arc<dyn StatusValue>>);

#[derive(Resource, Default, Clone)]
pub struct StatusBarStore {
    pub store: HashMap<Entity, Arc<dyn StatusValue>>,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatusBarPluginSystems {
    Sync,
}

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        debug!("StatusBarPlugin: build");

        app.init_resource::<StatusBarStore>();

        app.add_observer(insert_status_bar_item);

        app.add_systems(
            Update,
            (update_status_bar_item_values, update_status_bar_item_ui)
                .chain()
                .in_set(StatusBarPluginSystems::Sync),
        );
    }
}

// If you really need full, immediate read/write access to the world or resources, you can use an
// "exclusive system".
// WARNING: These will block all parallel execution of other systems until they finish, so they
// should generally be avoided if you want to maximize parallelism.
fn update_status_bar_item_values(world: &mut World) {
    let mut query =
        world.query_filtered::<(Entity, &StatusBarItemValueFnBoxed), With<StatusBarItemValue>>();
    let values: HashMap<_, _> = query
        .iter(world)
        .map(|(entity, value_fn)| (entity, value_fn(world)))
        .collect();

    let mut query =
        world.query_filtered::<(Entity, &mut StatusBarItemValue), With<StatusBarItemValue>>();
    for (entity, mut item) in query.iter_mut(world) {
        if let Some(value) = values.get(&entity) {
            **item = value.clone();
        }
    }
}

fn update_status_bar_item_ui(
    mut items: Query<(
        &StatusBarItemValue,
        &mut Text,
        &mut TextColor,
        &StatusBarItemColorFnBoxed,
    )>,
) {
    for (value, mut text, mut color, color_fn) in &mut items {
        **text = value
            .as_ref()
            .map_or_else(|| "N/A".to_string(), |v| v.to_string());

        if let Some(v) = value.as_ref() {
            let v: &dyn Any = v.as_ref();

            if let Some(new_color) = (color_fn)(Box::new(v)) {
                **color = new_color;
            }
        }
    }
}

fn insert_status_bar_item(
    add: On<Add, StatusBarItemMarker>,
    mut commands: Commands,
    q_item: Query<
        (
            &StatusBarItemIcon,
            &StatusBarItemPrefix,
            &StatusBarItemSuffix,
            &StatusBarItemValueFnBoxed,
            &StatusBarItemColorFnBoxed,
        ),
        With<StatusBarItemMarker>,
    >,
    root: Single<Entity, With<StatusBarRootMarker>>,
) {
    let entity = add.entity;
    trace!("insert_status_bar_item: entity {:?}", entity);

    let Ok((icon, prefix, suffix, value_fn, color_fn)) = q_item.get(entity) else {
        warn!(
            "insert_status_bar_item: entity {:?} not found in q_item",
            entity
        );
        return;
    };

    let root = root.into_inner();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new(format!("StatusBarItem: {}-{}", **prefix, **suffix)),
            Node {
                width: Val::Auto,
                height: Val::Px(24.0),
                margin: UiRect::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
            children![
                (
                    Name::new("StatusBarItemIcon"),
                    ImageNode {
                        image: (**icon).clone(),
                        ..default()
                    },
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        ..default()
                    },
                ),
                (
                    Name::new("StatusBarItemPrefix"),
                    Text::new((**prefix).clone()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                ),
                (
                    Name::new("StatusBarItemValue"),
                    StatusBarItemValue(None),
                    value_fn.clone(),
                    Text::new("N/A".to_string()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    color_fn.clone(),
                    TextColor(Color::WHITE),
                ),
                (
                    Name::new("StatusBarItemSuffix"),
                    Text::new((**suffix).clone()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                )
            ],
        ));
    });
}

pub fn status_fps_value_fn(
) -> impl Fn(&World) -> Option<Arc<dyn StatusValue>> + Send + Sync + 'static {
    move |world: &World| {
        let store = world.resource::<bevy::diagnostic::DiagnosticsStore>();
        store
            .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.average())
            .map(|v| v.round() as u32)
            .map(|fps| Arc::new(fps) as Arc<dyn StatusValue>)
    }
}

pub fn status_fps_color_fn() -> impl Fn(Box<&dyn Any>) -> Option<Color> + Send + Sync + 'static {
    move |value: Box<&dyn Any>| {
        let fps = (*value).downcast_ref::<u32>()?;
        let color = if *fps < 30 {
            Color::srgb(1.0, 0.0, 0.0)
        } else if *fps < 60 {
            Color::srgb(1.0, 1.0, 0.0)
        } else {
            Color::srgb(0.0, 1.0, 0.0)
        };
        Some(color)
    }
}

pub fn status_version_value_fn(
    version: impl Display + Clone + Send + Sync + 'static,
) -> impl Fn(&World) -> Option<Arc<dyn StatusValue>> + Send + Sync + 'static {
    move |_world: &World| Some(Arc::new(version.clone()) as Arc<dyn StatusValue>)
}

pub fn status_version_color_fn() -> impl Fn(Box<&dyn Any>) -> Option<Color> + Send + Sync + 'static
{
    move |_value: Box<&dyn Any>| Some(Color::srgb(1.0, 1.0, 1.0))
}
