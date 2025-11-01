pub mod despawn;
pub mod temp;
pub mod wasd;

pub mod prelude {
    pub use super::{despawn::prelude::*, temp::prelude::*, wasd::prelude::*};
}
