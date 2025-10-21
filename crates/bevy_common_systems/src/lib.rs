//! TODO: Add description in this crate

pub mod camera;
pub mod health;
pub mod helpers;
pub mod mesh;
pub mod meth;
pub mod projectiles;
pub mod transform;

pub mod prelude {
    pub use crate::camera::prelude::*;
    pub use crate::health::prelude::*;
    pub use crate::helpers::prelude::*;
    pub use crate::mesh::prelude::*;
    pub use crate::meth::prelude::*;
    pub use crate::projectiles::prelude::*;
    pub use crate::transform::prelude::*;
}
