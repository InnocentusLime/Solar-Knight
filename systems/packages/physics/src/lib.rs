use nalgebra::Vector2;

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
            force : Vector2::new(0.0f32, 0.0f32),
            velocity : Vector2::new(0.0f32, 0.0f32),
        }
    }
}

pub struct PhysicsSystem {
    env_friction : f32,
    resitution : f32,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        PhysicsSystem {
            env_friction : 0.24f32,
            resitution : 1.0f32,
        }
    }

    fn solve_collisions<Host, Observer>(
        &mut self, 
        observation : &mut Observation<Observer, Host>, 
    )
    where
        Host : Storage,
        Host::Object : ComponentAccess<PhysicsData> + ComponentAccess<Transform>,
        Observer : MutationObserver<Host>,
    {
        /* TODO */
        for colliding_obj_id in 0..observation.capacity() {
            // TODO optimize this with some data structure
            for collider_obj_id in 0..observation.capacity() {
                if (colliding_obj_id == collider_obj_id) { continue; }

                /* collision resolving code for the two objects */ 
            }
        }
    }


    fn tick<Host, Observer>(
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
                let phys = get_component_mut::<PhysicsData, _>(obj);
                if 
                    phys.velocity.magnitude() >= VECTOR_NORMALIZATION_RANGE
                {
                    phys.force -= self.env_friction * phys.velocity.magnitude() * phys.velocity;
                }
                let acceleration = phys.force / phys.mass;
                phys.velocity += dt.as_secs_f32() * acceleration; 
                phys.force = Vector2::new(0.0f32, 0.0f32);

                let dr = phys.velocity * dt.as_secs_f32() + acceleration / 2.0f32 * dt.as_secs_f32().powi(2);
                get_component_mut::<Transform, _>(obj).transform.translation.vector += dr;
            }
        )
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
        self.solve_collisions(observation);
        self.tick(observation, dt);
    }
}
