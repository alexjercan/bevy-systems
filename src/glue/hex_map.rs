use crate::{hexmap::prelude::*, noise::prelude::*};

impl ToNoisePoint<3> for HexCoord {
    fn to_point(&self) -> [f64; 3] {
        let q = self.x as f64;
        let r = self.y as f64;
        let s = -(q + r);

        [q, r, s]
    }
}
