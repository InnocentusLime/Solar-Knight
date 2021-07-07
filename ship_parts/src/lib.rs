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

mod ai {
    use super::*;

    pub fn shoot(
        me : &mut Ship,
        bullet_system : &mut crate::gun::BulletSystem,
        gun : usize,
    ) {
        me.guns[0].shoot(&me.core) 
        .map_or(
            (),
            |x| bullet_system.spawn(x)
        )
    }

    pub fn can_see(
        me : &Ship,
        target : Point2<f32>,
        view_angle : f32,
    ) -> bool {
        let dir_vec = target - me.core.pos;
        let ang = me.core.direction().angle(dir_vec);
        ang.0.abs() <= view_angle / 2.0f32
    }

    pub fn target_near(
        me : &Ship,
        target : Point2<f32>,
        coverage_radius : f32,
    ) -> bool {
        let dir_vec = (target - me.core.pos).normalize();
        dir_vec.magnitude() <= coverage_radius
    }

    pub fn rotate_towards(
        me : &mut Ship,
        target : Point2<f32>,
        angular_speed : f32,
        dt : Duration,
    ) -> bool {
        let dir_vec = (target - me.core.pos).normalize();
        let dir_vec = dir_vec.normalize();
        let ang = me.core.direction().angle(dir_vec);

        if abs_diff_ne!(ang.0, 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) {
            let (c, s) =
                if ang.0 > 0.0f32 {
                    ((angular_speed * dt.as_secs_f32()).cos(), (angular_speed * dt.as_secs_f32()).sin())
                } else {
                    ((angular_speed * dt.as_secs_f32()).cos(), -(angular_speed * dt.as_secs_f32()).sin())
                }
            ;
            me.core.set_direction(rotate_vector_ox(me.core.direction(), vec2(c, s)));
            false
        } else { true }
    }
}

fn no_ai(
    _me : &mut Ship,
    _others : &std_ext::ExtractResultMut<Ship>, 
    _bullet_system : &mut crate::gun::BulletSystem,
    _earth : &Earth,
    _dt : Duration,
) {}

fn turret_ai(
    me : &mut Ship,
    others : &std_ext::ExtractResultMut<Ship>, 
    bullet_system : &mut crate::gun::BulletSystem,
    _earth : &Earth,
    dt : Duration,
) {
    const TURRET_ROTATION_SPEED : f32 = std::f32::consts::TAU / 4.0f32;
    let target = others[0].core.pos;

    if ai::target_near(me, target, 1.5f32) {
        ai::rotate_towards(me, target, TURRET_ROTATION_SPEED, dt);
        if ai::can_see(me, target, std::f32::consts::PI / 8.0f32) {
            ai::shoot(me, bullet_system, 0);
        }
    }
}

/*
fn _ai(
    me : &mut Ship,
    others : &std_ext::ExtractResultMut<Ship>, 
    bullet_system : &mut crate::gun::BulletSystem,
    _earth : &Earth,
    dt : Duration,
)
*/

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
