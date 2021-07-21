use cgmath::{ abs_diff_ne, InnerSpace, vec2 };

use std::time::Duration;

use crate::storage_traits::{ Observation, MutationObserver };

pub struct PhysicsSystem {
    env_friction : f32,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        PhysicsSystem {
            env_friction : 0.24f32,
        }
    }

    pub fn update<Observer : MutationObserver>(
        &mut self, 
        observation : &mut Observation<Observer>, 
        dt : Duration
    ) {
        use crate::constants::VECTOR_NORMALIZATION_RANGE;

        observation.mutate_each(
            |c| {
                c.core.force = vec2(0.0f32, 0.0f32);

                let (core, engines, guns) = (&mut c.core, &mut c.engines, &mut c.guns);
                // TODO not part of the phys sys
                engines.iter_mut().for_each(|x| x.update(core, dt));
                guns.iter_mut().for_each(|x| x.update(core, dt));

                if 
                    abs_diff_ne!(c.core.velocity.magnitude(), 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) 
                {
                    c.core.force -= self.env_friction * c.core.velocity.magnitude() * c.core.velocity;
                }
                let acceleration = c.core.force / c.core.mass;
                c.core.pos += dt.as_secs_f32() * c.core.velocity + dt.as_secs_f32().powi(2) * acceleration / 2.0f32;
                c.core.velocity += dt.as_secs_f32() * acceleration;
            }
        )
    }
}
