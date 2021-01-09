use std::time::Duration;

use cgmath::{ Point2, EuclideanSpace, Angle, InnerSpace, Rad, Matrix4, vec2 };

use crate::duration_ext::*;

pub const EARTH_SUN_DISTANCE : f32 = 3.0f32;
pub const EARTH_LOOP_TIME : Duration = Duration::from_secs(40);
// Duration::as_secs_f32 is not a const fn yet (72440)
//pub const EARTH_ANGLE_SPEED : f32 = std::f32::consts::TAU / EARTH_LOOP_TIME.as_secs_f32();
pub const EARTH_ANGLE_SPEED : f32 = std::f32::consts::TAU / 40.0f32;

pub struct Earth {
    pos : Point2<f32>,
    timing : Duration,
}

impl Earth {
    pub fn new() -> Earth {
        Earth {
            pos : Point2 { x : EARTH_SUN_DISTANCE, y : 0.0f32 },
            timing : <Duration as DurationExt>::my_zero(),
        }
    }

    pub fn pos(&self) -> Point2<f32> { self.pos }

    pub fn update(&mut self, dt : Duration) {
        self.timing += dt;

        let phase = self.timing.as_secs_f32() * EARTH_ANGLE_SPEED;
        self.pos = 
            Point2 { 
                x : EARTH_SUN_DISTANCE * phase.cos(), 
                y : EARTH_SUN_DISTANCE * phase.sin(),
            }
        ;

        self.timing = self.timing.my_rem(EARTH_LOOP_TIME)
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_nonuniform_scale(0.3f32, 0.3f32, 1.0f32)
    }
}
