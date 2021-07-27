// TODO-FIXME ALL THE VECTORS IN SHIPS SHOULD BE CHECKED
// Last meeting it turned out that somewhere in the code there's
// a possibility to `NaN` or `Inf` the vectors which causes the 
// computations to collapse. This must be fixed.

pub mod engine;
pub mod gun;
pub mod render;
pub mod core;
pub mod earth;
pub mod collision_models;
pub mod constants;
pub mod storage_traits;
pub mod part_trait;
pub mod attachment;
pub mod ai_machine;
pub mod square_map;
pub mod physics;
pub mod storage;

pub use crate::core::Team;
pub use crate::earth::Earth;
pub use crate::gun::{ BulletSystem };
pub use crate::storage::{ Ship, Storage, MutableStorage };
pub use crate::storage_traits::*;

crate::declare_observers! {
    spawning_observers : {
        square_map : crate::square_map::SquareMap,
    },
    deletion_observers : {
        square_map : crate::square_map::SquareMap,
        attach_sys : crate::attachment::AttachmentSystem,
        bullet_sys : crate::gun::BulletSystem,
    },
    mutation_observers : {
        square_map : crate::square_map::SquareMap,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerTarget {
    None,
    Sun,
    Earth,
}
