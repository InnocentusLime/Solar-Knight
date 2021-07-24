use std::time::Duration;

use slab::Slab;
use glium::VertexBuffer;
use tinyvec::ArrayVec;
use tinyvec::array_vec;
use serde::{ Serialize, Deserialize };
use cgmath::{ Point2, Matrix4, EuclideanSpace, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

use cgmath_ext::rotate_vector_ox;

use crate::earth::Earth;
use crate::gun::BulletSystem;
use crate::storage::{ Ship, Storage };
use crate::constants::VECTOR_NORMALIZATION_RANGE;
use crate::storage_traits::{ Observation, MutationObserver };

#[derive(Clone, Copy, Debug)]
pub struct RoutineId(pub usize);

// TODO if everything goes well (all AI routines will be implementable like `turret_ai`, make 
// `is_target_close` and `is_target_in_sight` to accept `Ship`
mod ai {
    use std::time::Duration;
    use crate::storage::{ Ship };
    use cgmath::{ Vector2, Point2, InnerSpace, vec2 };
    use crate::ai_machine::rotate_vector_ox;

    #[inline]
    pub fn is_target_close(
        me : Point2<f32>,
        target : Point2<f32>,
        distance : f32,
    ) -> bool {
        let dir_vec = (target - me).normalize();
        dir_vec.magnitude() <= distance
    }

    #[inline]
    pub fn is_target_in_sight(
        me : Point2<f32>,
        direc : Vector2<f32>,
        target : Point2<f32>,
        view_angle : f32,
    ) -> bool {
        let dir_vec = target - me;
        let ang = direc.angle(dir_vec);
        ang.0.abs() <= view_angle / 2.0f32
    }

    #[inline]
    pub fn rotate_towards(
        me : &mut Ship,
        target : Point2<f32>,
        angular_speed : f32,
        dt : Duration,
    ) {
        let dir_vec = (target - me.core.pos).normalize();
        let ang = me.core.direction().angle(dir_vec);

        if ang.0.abs() > angular_speed * dt.as_secs_f32() {
            let (c, s) =
                if ang.0 > 0.0f32 {
                    ((angular_speed * dt.as_secs_f32()).cos(), (angular_speed * dt.as_secs_f32()).sin())
                } else {
                    ((angular_speed * dt.as_secs_f32()).cos(), -(angular_speed * dt.as_secs_f32()).sin())
                } 
            ;
            me.core.set_direction(rotate_vector_ox(me.core.direction(), vec2(c, s)));
        } else { me.core.set_direction(dir_vec); }
    }
}

fn turret_ai(
    me : usize,
    storage : &mut Observation<crate::MutationObserverPack>,
    earth : &Earth,
    bullet_sys : &mut crate::gun::BulletSystem,
    dt : Duration
) {
    let target = storage.get(0).unwrap().core.pos;

    let (target_close, can_see) = 
        storage.get(me).map(|x|{
            let (direc, pos) = (x.core.direction(), x.core.pos);
            (
                ai::is_target_close(pos, target, 1.5f32),
                ai::is_target_in_sight(pos, direc, target, std::f32::consts::TAU / 8.0f32)
            )
        }).unwrap()
    ;

    storage.mutate(me, |ship| {
        if target_close {
            if can_see {
                bullet_sys.shoot_from_gun_ship(ship, me, 0);
            } else {
                ai::rotate_towards(ship, target, std::f32::consts::TAU / 4.0f32, dt)
            }
        }
    });
}

fn heavy_body_ai(
    me : usize,
    storage : &mut Observation<crate::MutationObserverPack>,
    earth : &Earth,
    bullet_sys : &mut crate::gun::BulletSystem,
    dt : Duration
) {
    storage.mutate(me, |ship| {
        ai::rotate_towards(ship, earth.pos(), std::f32::consts::TAU / 16.0, dt)
    });
}

fn fly_ai(
    me : usize,
    storage : &mut Observation<crate::MutationObserverPack>,
    earth : &Earth,
    bullet_sys : &mut crate::gun::BulletSystem,
    dt : Duration
) {
    let target = storage.get(0).unwrap().core.pos;

    storage.mutate(me, |ship| {
        if ai::is_target_close(ship.core.pos, target, 0.8f32) {
            ship.engines[0].set_level(1);
        } else { ship.engines[0].set_level(3); }    
        ai::rotate_towards(ship, target, std::f32::consts::TAU * 2.0, dt)
    });
}

// TODO Error codes
// TODO State for AI (Consider only when the need shows up)
type AiRoutine = 
    fn(
        usize,
        &mut Observation<crate::MutationObserverPack>,
        &Earth,
        &mut crate::gun::BulletSystem,
        Duration
    )
;

//#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct AiMachine {
    routines : Vec<(String, AiRoutine)>,
}

impl AiMachine {
    pub fn new() -> Self {
        AiMachine {
            routines : vec![
                ("turret AI".to_owned(), turret_ai),
                ("heavy's body AI".to_owned(), heavy_body_ai),
                ("fly AI".to_owned(), fly_ai),
            ],
        }
    }

    pub fn get_ai_name(
        &self,
        id : usize
    ) -> Option<&String> {
        self.routines.get(id).map(|x| &x.0)
    }

    pub fn update(
        &self, 
        storage : &mut Observation<crate::MutationObserverPack>, 
        earth : &Earth,
        bullet_system : &mut BulletSystem, 
        dt : Duration
    ) {
        //TODO read the TODO above `AiRoutine`
        for i in 0..storage.capacity() {
            let (alive, think) = {
                match storage.get(i) {
                    Some(ship) => (ship.core.is_alive(), ship.think),
                    None => continue,
                }
            };
            
            if alive {
                if let Some(routine_id) = think {
                    (self.routines[routine_id.0].1)(
                        i,
                        storage,
                        earth,
                        bullet_system,
                        dt,
                    )
                }
            }
        }
    }
}

