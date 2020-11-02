use cgmath::{ Point2, EuclideanSpace, Angle, InnerSpace, Rad, Matrix4, vec2 };

pub const EARTH_PHASE_DELTA : f32 = 0.02f32;
pub const EARTH_SUN_DISTANCE : f32 = 3.0f32;
pub const EARTH_POSITION : (f32, f32) = (EARTH_SUN_DISTANCE, 0.0f32);

pub struct Earth {
    pos : Point2<f32>,
}

impl Earth {
    pub fn new() -> Earth {
        Earth {
            pos : Point2 { x : EARTH_POSITION.0, y : EARTH_POSITION.1 },
        }
    }

    pub fn update(&mut self) {
        let start_vec = vec2(EARTH_POSITION.0, EARTH_POSITION.1);
        let curr_vec = self.pos.to_vec();

        let phase = start_vec.angle(curr_vec);
        let new_phase = phase + Rad(EARTH_PHASE_DELTA);
        let (s, c) = new_phase.sin_cos();
        let new_pos = Point2 { x : EARTH_SUN_DISTANCE * c, y : EARTH_SUN_DISTANCE * s };

        self.pos = new_pos;
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_nonuniform_scale(0.3f32, 0.3f32, 1.0f32)
    }
}
