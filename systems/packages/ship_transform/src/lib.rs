use cgmath::{ Matrix3, Matrix4, InnerSpace, EuclideanSpace, Point2, Vector2, assert_abs_diff_eq, vec2 };

use cgmath_ext::matrix3_from_translation;

pub const VECTOR_NORMALIZATION_RANGE : f32 = 0.0001f32;
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos : Point2<f32>,
    direction : Vector2<f32>,
}

impl Transform {
    pub const fn new(pos : Point2<f32>) -> Self {
        Transform {
            pos,
            direction : vec2(0.0f32, 1.0f32),
        }
    }

    #[inline]
    pub fn model_mat(&self, size : (f32, f32)) -> Matrix4<f32> {
        let direction = self.direction();

        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * 
        Matrix4::new(
            direction.y, -direction.x, 0.0f32, 0.0f32,
            direction.x, direction.y, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ) * 
        Matrix4::from_nonuniform_scale(size.0, size.1, 1.0f32)
    }

    #[inline]
    pub fn transform(&self) -> Matrix3<f32> {
        matrix3_from_translation(self.pos.to_vec()) *
        Matrix3::new(
            self.direction.y, -self.direction.x, 0.0f32,
            self.direction.x, self.direction.y, 0.0f32,
            0.0f32, 0.0f32, 1.0f32,
        )
    }

    #[inline]
    pub fn direction(&self) -> Vector2<f32> {
        self.direction
    }

    /// # Description
    /// Changes the direction of the ship.
    ///
    /// # Panics
    /// Panics when the direction argument is not a unit vector.
    #[inline]
    pub fn set_direction(&mut self, direction : Vector2<f32>) {
        assert!(direction.x.is_finite()); assert!(direction.y.is_finite());
        assert_abs_diff_eq!(direction.magnitude(), 1.0f32, epsilon = VECTOR_NORMALIZATION_RANGE);
        self.direction = direction;
    } 

    #[inline]
    pub fn set_direction_angle(&mut self, ang : f32) {
        let (s, c) = (std::f32::consts::FRAC_PI_2 - ang).sin_cos();
        self.set_direction(vec2(c, s));
    }
}
