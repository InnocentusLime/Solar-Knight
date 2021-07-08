// TODO-FIXME ALL THE VECTORS IN SHIPS SHOULD BE CHECKED
// Last meeting it turned out that somewhere in the code there's
// a possibility to `NaN` or `Inf` the vectors which causes the 
// computations to collapse. This must be fixed.

pub mod engine;
pub mod gun;
pub mod core;
pub mod earth;
pub mod collision_models;
pub mod constants;
pub mod storage_traits;
pub mod part_trait;
pub mod attachment;

pub use crate::core::Team;
pub use crate::earth::Earth;
pub use crate::gun::BulletSystem;
pub use crate::storage_traits::{ Ship, Battlefield, RoutineId };       

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

fn test_render(
    me : &Ship,
    buff : &mut SpriteDataWriter,
) {
    let m = me.model_mat((0.1f32, 0.1f32));
    //dbg!(i); dbg!(m);
    
    let color : [f32; 4];
    if me.core.team() == Team::Hive { color = [1.0f32, 0.01f32, 0.01f32, 1.0f32] }
    else { color = [1.0f32; 4] }
            
    let dat =
        SpriteData {
            mat_col1 : m.x.into(),
            mat_col2 : m.y.into(),
            mat_col3 : m.z.into(),
            mat_col4 : m.w.into(),
            texture_bottom_left : [0.0f32, 0.0f32],
            width_height : [1.0f32, 1.0f32],
            color : color,
        }
    ;

    buff.put(dat);
}
