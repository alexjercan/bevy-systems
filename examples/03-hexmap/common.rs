use bevy::prelude::*;
use hexx::*;

#[derive(Component, Debug, Deref, DerefMut)]
pub struct HexCoord(pub Hex);

impl From<&HexCoord> for [f64; 3] {
    fn from(value: &HexCoord) -> Self {
        let q = value.x as f64;
        let r = value.y as f64;
        let s = -(q + r);

        [q, r, s]
    }
}

impl From<Hex> for HexCoord {
    fn from(hex: Hex) -> Self {
        Self(hex)
    }
}

impl Into<Hex> for &HexCoord {
    fn into(self) -> Hex {
        self.0
    }
}
