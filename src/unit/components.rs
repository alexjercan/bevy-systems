use std::collections::VecDeque;

use bevy::prelude::*;
use hexx::*;

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Deref, DerefMut)]
pub struct UnitCoord(pub Hex);

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq, Reflect, Deref, DerefMut)]
pub struct UnitTarget(pub Hex);

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq, Reflect, Deref, DerefMut)]
pub struct UnitPath(pub VecDeque<Hex>);
