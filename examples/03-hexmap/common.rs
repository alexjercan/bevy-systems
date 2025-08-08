use bevy::prelude::*;
use hexx::*;
use systems::{hexmap::prelude::*, noise::prelude::*};

#[derive(Component, Debug, Deref, DerefMut)]
pub struct HexCoord(pub Hex);

impl ToNoisePoint<3> for HexCoord {
    fn to_point(&self) -> [f64; 3] {
        let q = self.x as f64;
        let r = self.y as f64;
        let s = -(q + r);

        [q, r, s]
    }
}

impl FromHex for HexCoord {
    fn from_hex(hex: Hex) -> Self {
        Self(hex)
    }
}
