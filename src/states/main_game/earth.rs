use cgmath::{ Point2, EuclideanSpace, Angle, InnerSpace, Rad, Matrix4, vec2 };

pub const EARTH_PHASE_COUNT : u64 = 2000;
pub const EARTH_SUN_DISTANCE : f32 = 3.0f32;
pub const EARTH_PHASE_DELTA : f32 = std::f32::consts::TAU / (EARTH_PHASE_COUNT as f32);

pub struct Earth {
    pos : Point2<f32>,
    phase : u64,
}

impl Earth {
    pub fn new() -> Earth {
        Earth {
            pos : Point2 { x : EARTH_SUN_DISTANCE, y : 0.0f32 },
            phase : 0,
        }
    }

    pub fn update(&mut self) {
        self.phase = (self.phase + 1) % EARTH_PHASE_COUNT;
        self.pos = 
            Point2 { 
                x : EARTH_SUN_DISTANCE * (self.phase as f32 * EARTH_PHASE_DELTA).cos(), 
                y : EARTH_SUN_DISTANCE * (self.phase as f32 * EARTH_PHASE_DELTA).sin(),
            }
        ;
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_nonuniform_scale(0.3f32, 0.3f32, 1.0f32)
    }
}
