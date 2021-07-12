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

pub use crate::core::Team;
pub use crate::earth::Earth;
pub use crate::gun::BulletSystem;
pub use crate::storage_traits::{ Ship, Battlefield };       

use std::time::Duration;

use lazy_static::lazy_static;
use cgmath::{ Vector2, Point2, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

use crate::core::Core;
use crate::engine::Engine;
use crate::gun::{ Gun, Bullet };
use crate::part_trait::ShipPart;
use crate::constants::VECTOR_NORMALIZATION_RANGE;

use cgmath_ext::rotate_vector_ox;

use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;

