use cgmath::{ Vector2, InnerSpace, VectorSpace };

pub trait VectorExt : VectorSpace + InnerSpace + Sized {
    fn mag_clamp(self, mag : <Self as VectorSpace>::Scalar) -> Self;
}

impl VectorExt for Vector2<f32> {
    fn mag_clamp(self, mag : f32) -> Self {
        assert!(mag > 0.0f32);
        let my_mag = self.magnitude();
        if my_mag > mag { mag * self.normalize() }
        else { self }
    }
}
