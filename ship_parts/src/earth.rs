use std::time::Duration;

use nalgebra::{ Point2, Point3, Vector3, Matrix4 };

use std_ext::*;

pub struct Earth {
    pos : Point2<f32>,
    timing : Duration,
}

impl Earth {
    pub const SUN_DISTANCE : f32 = 3.0f32;
    pub const LOOP_TIME : Duration = Duration::from_secs(80);
    // Duration::as_secs_f32 is not a const fn yet (72440)
    //pub const EARTH_ANGLE_SPEED : f32 = std::f32::consts::TAU / EARTH_LOOP_TIME.as_secs_f32();
    pub const ANGLE_SPEED : f32 = std::f32::consts::TAU / 80.0f32;

    pub fn new() -> Earth {
        Earth {
            pos : Point2::new(Self::SUN_DISTANCE, 0.0f32),
            timing : <Duration as DurationExt>::my_zero(),
        }
    }

    pub fn pos(&self) -> Point2<f32> { self.pos }

    pub fn update(&mut self, dt : Duration) {
        self.timing += dt;

        let phase = self.timing.as_secs_f32() * Self::ANGLE_SPEED;
        self.pos = Self::SUN_DISTANCE * Point2::new(phase.cos(), phase.sin());

        self.timing = self.timing.my_rem(Self::LOOP_TIME)
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.pos.coords.push(0.0f32)) *
        Matrix4::new_nonuniform_scaling_wrt_point(
            &Vector3::new(0.3f32, 0.3f32, 1.0f32),
            &Point3::new(0.0f32, 0.0f32, 0.0f32)
        )
    }
}
