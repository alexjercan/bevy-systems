//! TODO: Add description in this crate

pub mod sections;
pub mod projectiles;
pub mod attachments;

pub mod prelude {
    pub use super::sections::prelude::*;
    pub use super::projectiles::prelude::*;
    pub use super::attachments::prelude::*;
}
