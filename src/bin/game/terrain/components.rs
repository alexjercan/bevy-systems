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

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseHeight(f64);

impl From<f64> for HexNoiseHeight {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseTemperature(f64);

impl From<f64> for HexNoiseTemperature {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct HexNoiseHumidity(f64);

impl From<f64> for HexNoiseHumidity {
    fn from(noise: f64) -> Self {
        Self(noise)
    }
}
