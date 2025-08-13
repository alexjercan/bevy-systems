//! Tilemap module for different types of tilemaps
//!
//! The tilemaps work in an ECS style, where each tile is a separate entity. The tiles are grouped
//! into chunks, which are also entities. The tilemap exposes an event to discover tiles in a
//! certain radius, and a system to handle the discovery of tiles and chunks.

pub mod hexmap;

pub mod prelude {
    pub use super::hexmap::prelude::*;
}
