use std::time::Duration;

//use serde::{ Serialize, Deserialize };

use cgmath_ext::rotate_vector_ox;
use systems::teams::Team;
use systems::ship_engine::Engines;
use systems::hp_system::HpInfo;
use systems::ship_transform::Transform;
use systems::ship_gun::{ Guns, BulletSystem };
use systems::systems_core::{ Storage, Observation, MutationObserver, ComponentAccess, get_component, get_component_mut };

use crate::earth::Earth;

// TODO Display impl
#[derive(Clone, Copy, Debug)]
pub enum AiTag {
    None,
    Turret,
    HeavyBody,
    Fly,
}

// TODO if everything goes well (all AI routines will be implementable like `turret_ai`, make 
// `is_target_close` and `is_target_in_sight` to accept `Ship`
mod ai {
    use std::time::Duration;
    use cgmath::{ Point2, InnerSpace, vec2 };
    use crate::ai_machine::rotate_vector_ox;

    use super::{ Transform, ComponentAccess, get_component, get_component_mut };

    #[inline]
    pub fn is_target_close<Obj : ComponentAccess<Transform>>(
        obj : &Obj,
        target : Point2<f32>,
        distance : f32,
    ) -> bool {
        let pos = get_component::<Transform, _>(obj).pos;
        let dir_vec = (target - pos).normalize();
        dir_vec.magnitude() <= distance
    }

    #[inline]
    pub fn is_target_in_sight<Obj : ComponentAccess<Transform>>(
        obj : &Obj,
        target : Point2<f32>,
        view_angle : f32,
    ) -> bool {
        let transform = get_component::<Transform, _>(obj);
        let dir_vec = target - transform.pos;
        let ang = transform.direction().angle(dir_vec);
        ang.0.abs() <= view_angle / 2.0f32
    }

    #[inline]
    pub fn rotate_towards<Obj : ComponentAccess<Transform>>(
        obj : &mut Obj,
        target : Point2<f32>,
        angular_speed : f32,
        dt : Duration,
    ) {
        let transform = get_component_mut(obj);

        let dir_vec = (target - transform.pos).normalize();
        let ang = transform.direction().angle(dir_vec);

        if ang.0.abs() > angular_speed * dt.as_secs_f32() {
            let (c, s) =
                if ang.0 > 0.0f32 {
                    ((angular_speed * dt.as_secs_f32()).cos(), (angular_speed * dt.as_secs_f32()).sin())
                } else {
                    ((angular_speed * dt.as_secs_f32()).cos(), -(angular_speed * dt.as_secs_f32()).sin())
                } 
            ;
            transform.set_direction(rotate_vector_ox(transform.direction(), vec2(c, s)));
        } else { transform.set_direction(dir_vec); }
    }
}

fn turret_ai<Host, Observer>(
    me : usize,
    bullet_system : &mut BulletSystem,
    storage : &mut Observation<Observer, Host>,
    _earth : &Earth,
    dt : Duration
) 
where
    Host : Storage,
    Host::Object : ComponentAccess<Transform> + ComponentAccess<Guns> + ComponentAccess<Team>,
    Observer : MutationObserver<Host>,
{
    let target = get_component::<Transform, _>(storage.get(0).unwrap()).pos;

    storage.mutate(me, |obj, _| {
        if ai::is_target_close(obj, target, 1.5f32) {
            if ai::is_target_in_sight(obj, target, std::f32::consts::TAU / 8.0f32) {
                bullet_system.shoot_from_gun_ship(obj, me, 0);
            }
            ai::rotate_towards(obj, target, std::f32::consts::TAU / 4.0f32, dt)
        }
    });
}

fn heavy_body_ai<Host, Observer>(
    me : usize,
    _bullet_system : &mut BulletSystem,
    storage : &mut Observation<Observer, Host>,
    earth : &Earth,
    dt : Duration
) 
where
    Host : Storage,
    Host::Object : ComponentAccess<Transform>,
    Observer : MutationObserver<Host>,
{
    storage.mutate(me, |obj, _| {
        ai::rotate_towards(obj, earth.pos(), std::f32::consts::TAU / 16.0, dt)
    });
}

fn fly_ai<Host, Observer>(
    me : usize,
    _bullet_system : &mut BulletSystem,
    storage : &mut Observation<Observer, Host>,
    _earth : &Earth,
    dt : Duration
) 
where
    Host : Storage,
    Host::Object : ComponentAccess<Transform> + ComponentAccess<Engines>,
    Observer : MutationObserver<Host>,
{
    let target = get_component::<Transform, _>(storage.get(0).unwrap()).pos;

    storage.mutate(me, |obj, _| {
        if ai::is_target_close(obj, target, 0.8f32) {
            get_component_mut::<Engines, _>(obj).engines[0].set_level(1);
        } else { 
            get_component_mut::<Engines, _>(obj).engines[0].set_level(3); 
        }
        ai::rotate_towards(obj, target, std::f32::consts::TAU * 2.0, dt)
    });
}

//#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct AiMachine {}

impl AiMachine {
    pub fn new() -> Self {
        AiMachine {}
    }

    fn update_obj<Host, Observer>(
        &self,
        id : usize,
        tag : AiTag,
        storage : &mut Observation<Observer, Host>, 
        earth : &Earth,
        bullet_system : &mut BulletSystem, 
        dt : Duration
    )
    where
        Host : Storage,
        Host::Object : ComponentAccess<Transform> + ComponentAccess<Engines> + ComponentAccess<Guns> + ComponentAccess<Team>,
        Observer : MutationObserver<Host>,
    {
        match tag {
            AiTag::None => (),
            AiTag::Turret => turret_ai(id, bullet_system, storage, earth, dt),
            AiTag::HeavyBody => heavy_body_ai(id, bullet_system, storage, earth, dt),
            AiTag::Fly => fly_ai(id, bullet_system, storage, earth, dt),
        }
    }

    pub fn update<Host, Observer>(
        &self, 
        storage : &mut Observation<Observer, Host>, 
        earth : &Earth,
        bullet_system : &mut BulletSystem, 
        dt : Duration
    ) 
    where
        Host : Storage,
        Host::Object : ComponentAccess<Transform> + ComponentAccess<Engines> + ComponentAccess<Guns> + ComponentAccess<AiTag> + ComponentAccess<HpInfo> + ComponentAccess<Team>,
        Observer : MutationObserver<Host>,
    {
        //TODO read the TODO above `AiRoutine`
        for i in 0..storage.capacity() {
            let (alive, think) = {
                match storage.get(i) {
                    Some(obj) => 
                        (
                            get_component::<HpInfo, _>(obj).is_alive(), 
                            *get_component::<AiTag, _>(obj), 
                        )
                    ,
                    None => continue,
                }
            };
            
            if alive {
                self.update_obj(
                    i,
                    think,
                    storage,
                    earth,
                    bullet_system,
                    dt,
                )
            }
        }
    }
}

