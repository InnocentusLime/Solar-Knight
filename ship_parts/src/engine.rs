use cgmath_ext::rotate_vector_oy;
use crate::constants::VECTOR_NORMALIZATION_RANGE;
use cgmath::{ Vector2, InnerSpace, assert_abs_diff_eq, vec2 };

#[derive(Clone, Copy, Debug)]
pub struct Engine {
    direction : Vector2<f32>,
    max_lvl : u16,
    force_mul : f32,
    current_lvl : u16
}

impl Engine {
    pub fn new(
        direction : Vector2<f32>,
        max_lvl : u16,
        force_mul : f32,
        start_lvl : u16,
    ) -> Self {
        assert!(start_lvl <= max_lvl);
        assert_abs_diff_eq!(direction.magnitude(), 1.0f32, epsilon=VECTOR_NORMALIZATION_RANGE);

        Engine {
            direction,
            max_lvl,
            force_mul,
            current_lvl : start_lvl,
        }
    }
            
    #[inline]
    pub fn increase_speed(&mut self) {
        self.current_lvl = (self.current_lvl + 1).min(self.max_lvl);
    }

    #[inline]
    pub fn decrease_speed(&mut self) {
        self.current_lvl = self.current_lvl.saturating_sub(1); 
    }
    
    #[inline]
    pub fn update(&mut self, core : &mut crate::core::Core, _dt : std::time::Duration) {
        let direction = rotate_vector_oy(core.direction(), self.direction);
        core.force += (self.force_mul * self.current_lvl as f32) * direction;
    }
}

impl Default for Engine {
    fn default() -> Engine {
        Engine::new(vec2(0.0f32, 1.0f32), 0, 0.0f32, 0)
    }
}

#[macro_export]
macro_rules! exponential_decrease_curve {
    ($base:expr) => { 
        |t : f32, dt : f32, curr : f32, end : f32 | { 
            let frac_div = $base.powf(t) - 1.0f32;
            if frac_div < std::f32::EPSILON { end }
            else {
                let frac = ($base.powf((t - dt).max(0.0f32)) - 1.0f32) / ($base.powf(t) - 1.0f32);
                ((curr - end) * frac + end) 
            }
        }
    }
}
