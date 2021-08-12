use cgmath::{ Vector2, abs_diff_ne, InnerSpace, vec2 };

use std::time::Duration;

use ship_transform::Transform;
use systems_core::{ get_component_mut, ComponentAccess, Storage, MutationObserver, Observation };

pub const VECTOR_NORMALIZATION_RANGE : f32 = 0.0001f32;
#[derive(Clone, Copy, Debug)]
pub struct PhysicsData {
    pub mass : f32,
    pub force : Vector2<f32>,
    pub velocity : Vector2<f32>,
}

impl PhysicsData {
    #[inline]
    pub fn new(mass : f32) -> Self {
        PhysicsData {
            mass,
            force : vec2(0.0f32, 0.0f32),
            velocity : vec2(0.0f32, 0.0f32),
        }
    }
}

pub struct PhysicsSystem {
    env_friction : f32,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        PhysicsSystem {
            env_friction : 0.24f32,
        }
    }

    pub fn update<Host, Observer>(
        &mut self, 
        observation : &mut Observation<Observer, Host>, 
        dt : Duration
    ) 
    where
        Host : Storage,
        Host::Object : ComponentAccess<PhysicsData> + ComponentAccess<Transform>,
        Observer : MutationObserver<Host>,
    {
        observation.mutate_each(
            |obj, _| {
                // TODO not part of the phys sys
                //engines.iter_mut().for_each(|x| x.update(core, dt));
                //guns.iter_mut().for_each(|x| x.update(core, dt));

                let phys = get_component_mut::<PhysicsData, _>(obj);
                if 
                    abs_diff_ne!(phys.velocity.magnitude(), 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) 
                {
                    phys.force -= self.env_friction * phys.velocity.magnitude() * phys.velocity;
                }
                let acceleration = phys.force / phys.mass;
                phys.velocity += dt.as_secs_f32() * acceleration; 
                phys.force = vec2(0.0f32, 0.0f32);

                let dr = phys.velocity * dt.as_secs_f32() + acceleration / 2.0f32 * dt.as_secs_f32().powi(2);
                get_component_mut::<Transform, _>(obj).pos += dr;
            }
        )
    }
}
