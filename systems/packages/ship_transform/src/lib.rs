use nalgebra::{ Translation, Isometry2, Matrix4, Point2, Point3, Vector2, Vector3, UnitComplex };
use std::time::Duration;

pub const VECTOR_NORMALIZATION_RANGE : f32 = 0.0001f32;
// TODO store scale
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub transform : Isometry2<f32>,
    pub origin_offset : Vector2<f32>,
}

impl Transform {
    pub const fn new_with_origin(transform : Isometry2<f32>, origin_offset : Vector2<f32>) -> Self {
        Transform {
            transform,
            origin_offset,
        }
    }
    
    pub const fn new(transform : Isometry2<f32>) -> Self {
        Transform {
            transform,
            origin_offset : Vector2::new(0.0f32, 0.0f32),
        }
    }

    #[inline]
    pub fn model_mat(&self, (width, height) : (f32, f32)) -> Matrix4<f32> {
        let full_tf = self.full_transform();

        let mut mat =
            full_tf.rotation
            .to_rotation_matrix()
            .matrix()
            .fixed_resize(0.0f32)
        ;
        mat[(2, 2)] = 1.0f32;
        mat[(3, 3)] = 1.0f32;
        
        Translation { vector : full_tf.translation.vector.push(0.0f32) }.to_homogeneous() *
        mat *
        Matrix4::new_nonuniform_scaling_wrt_point(
            &Vector3::new(width, height, 1.0f32),
            &Point3::new(0.0f32, 0.0f32, 0.0f32)
        )
    }

    #[inline]
    pub fn full_transform(&self) -> Isometry2<f32> {
        self.transform * Translation { vector : self.origin_offset }
    } 
    
    #[inline]
    pub fn set_direction_angle(&mut self, ang : f32) {
        self.transform = 
            Isometry2::new(
                self.transform.translation.vector,
                ang
            )
        ;
    }

    #[inline]
    pub fn set_direction(
        &mut self,
        directing_vec : Vector2<f32>,
    ) {
        self.transform.rotation = UnitComplex::rotation_between(&Vector2::x(), &directing_vec) 
    }

    #[inline]
    pub fn point_at(
        &mut self,
        target : Point2<f32>,
    ) {
        self.set_direction(target.coords - self.transform.translation.vector)
    }

    #[inline]
    pub fn rotate_towards(
        &mut self,
        target : Point2<f32>,
        angular_speed : f32,
        dt : Duration,
    ) {
        let ang = self.angle_to(target);
        let ang_val = ang.angle();

        if ang_val.abs() > angular_speed * dt.as_secs_f32() {
            self.transform.rotation *= UnitComplex::new(angular_speed * dt.as_secs_f32() * ang_val.signum())
        } else { self.transform.rotation *= ang }
    }

    #[inline]
    pub fn move_in_direction(
        &mut self,
        amount : f32,
    ) {
        self.transform.translation.vector += self.transform.rotation.transform_vector(&(amount * Vector2::x()));
    }

    #[inline]
    pub fn rotation(&self) -> UnitComplex<f32> {
        self.transform.rotation
    }

    #[inline]
    pub fn position(&self) -> Point2<f32> {
        self.transform.translation.vector.into()
    }

    #[inline]
    pub fn angle_to(
        &self,
        target : Point2<f32>,
    ) -> UnitComplex<f32> {
        let target_vec = target.coords - self.transform.translation.vector;
        let dir_vec = self.transform.rotation.transform_vector(&Vector2::x());
        UnitComplex::rotation_between(&dir_vec, &target_vec)
    }
}
