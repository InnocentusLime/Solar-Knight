use cgmath::{ BaseFloat, Angle, Vector2, Matrix3, Rad };

pub fn matrix3_from_translation<S : BaseFloat>(v : Vector2<S>) -> Matrix3<S> {
    Matrix3::new(
        S::one(), S::zero(), S::zero(),
        S::zero(), S::one(), S::zero(),
        v.x, v.y, S::one(),
    )
}

pub fn matrix3_from_angle<S : BaseFloat, A : Into<Rad<S>>>(theta : A) -> Matrix3<S> {
    let (s, c) = theta.into().sin_cos();

    Matrix3::new(
        c, s, S::zero(),
        -s, c, S::zero(),
        S::zero(), S::zero(), S::one(),
    )
}
