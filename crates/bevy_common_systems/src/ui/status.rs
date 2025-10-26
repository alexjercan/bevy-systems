//! The Status plugin provides a simple status bar UI for displaying different metrics.
//! The idea is it should be easy to add what metrics to display via generic components
//! Then you update the components and the ststaus bar updates automatically.

use std::sync::Arc;

use bevy::{platform::collections::HashMap, prelude::*};

pub mod prelude {
    pub use super::status_bar;
    pub use super::status_bar_item;
    pub use super::StatusBarItemConfig;
    pub use super::StatusBarPlugin;
    pub use super::StatusBarRootConfig;
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

/// The StatusBarItemConfig component defines a single item in the status bar.
#[derive(Debug, Clone, Default)]
pub struct StatusBarItemConfig<F>
where
    F: Fn(&World) -> Option<u32> + Send + Sync + 'static,
{
    pub icon: Option<Handle<Image>>,
    pub value_fn: F,
    pub label: String,
    pub mapping: Vec<(Option<u32>, Color)>,
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct StatusBarItemIcon(pub Handle<Image>);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct StatusBarItemLabel(pub String);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct StatusBarItemMapping(pub Vec<(Option<u32>, Color)>);

#[derive(Component, Clone, Deref, DerefMut)]
pub struct StatusBarItemValueFnBoxed(pub Arc<dyn Fn(&World) -> Option<u32> + Send + Sync>);

pub fn status_bar_item<F>(config: StatusBarItemConfig<F>) -> impl Bundle
where
    F: Fn(&World) -> Option<u32> + Send + Sync + 'static,
{
    (
        Name::new("StatusBarItem"),
        StatusBarItemMarker,
        StatusBarItemIcon(config.icon.unwrap_or_default()),
        StatusBarItemLabel(config.label),
        StatusBarItemMapping(config.mapping),
        StatusBarItemValueFnBoxed(Arc::new(config.value_fn)),
    )
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct StatusBarItemValue(pub Option<u32>);

#[derive(Resource, Default, Clone, Debug)]
pub struct StatusBarStore {
    pub store: HashMap<Entity, u32>,
}

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusBarStore>();

        app.add_observer(insert_status_bar_item);

        app.add_systems(
            Update,
            (update_status_bar_items, update_status_bar_item).chain(),
        );
    }
}

// If you really need full, immediate read/write access to the world or resources, you can use an
// "exclusive system".
// WARNING: These will block all parallel execution of other systems until they finish, so they
// should generally be avoided if you want to maximize parallelism.
fn update_status_bar_items(world: &mut World) {
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

fn update_status_bar_item(
    mut items: Query<(
        &StatusBarItemValue,
        &mut Text,
        &mut TextColor,
        &StatusBarItemMapping,
    )>,
) {
    for (value, mut text, mut color, mapping) in &mut items {
        **text = value.map_or_else(|| "N/A".to_string(), |v| v.to_string());

        let new_color = value
            .and_then(|val| {
                mapping
                    .iter()
                    .find_map(|(threshold, map_color)| match threshold {
                        Some(thresh) if val <= *thresh => Some(*map_color),
                        None => Some(*map_color),
                        _ => None,
                    })
            })
            .unwrap_or(Color::WHITE);

        **color = new_color;
    }
}

fn insert_status_bar_item(
    add: On<Add, StatusBarItemMarker>,
    mut commands: Commands,
    q_item: Query<
        (
            &StatusBarItemIcon,
            &StatusBarItemLabel,
            &StatusBarItemValueFnBoxed,
            &StatusBarItemMapping,
        ),
        With<StatusBarItemMarker>,
    >,
    root: Single<Entity, With<StatusBarRootMarker>>,
) {
    let entity = add.entity;
    debug!("Inserting UI element for status bar item {:?}", entity);
    let Ok((icon, label, value_fn, mapping)) = q_item.get(entity) else {
        error!(
            "StatusBarItem entity {:?} missing required components",
            entity
        );
        return;
    };

    let root = root.into_inner();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new(format!("StatusBarItem: {}", **label)),
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
                    Name::new("StatusBarItemValue"),
                    StatusBarItemValue(None),
                    value_fn.clone(),
                    Text::new(format!("N/A")),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    mapping.clone(),
                ),
                (
                    Name::new("StatusBarItemLabel"),
                    Text::new((**label).clone()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                )
            ],
        ));
    });
}
