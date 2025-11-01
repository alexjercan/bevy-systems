pub mod temp;
pub mod wasd;
pub mod despawn;

pub mod prelude {
    pub use super::{temp::prelude::*, wasd::prelude::*, despawn::prelude::*};
}
