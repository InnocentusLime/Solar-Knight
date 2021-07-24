use cgmath::{ BaseFloat, Angle, Vector2, Matrix3, Rad, vec2 };

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

#[inline]
pub fn rotate_vector_ox<S : BaseFloat>(v : Vector2<S>, rot : Vector2<S>) -> cgmath::Vector2<S> {
    cgmath::vec2(
        v.x * rot.x - v.y * rot.y,
        v.x * rot.y + v.y * rot.x
    )
}

#[inline]
pub fn rotate_vector_oy<S : BaseFloat>(v : Vector2<S>, rot : Vector2<S>) -> cgmath::Vector2<S> {
    rotate_vector_ox(v, vec2(rot.y, -rot.x))
}

#[inline]
pub fn rotate_vector_angle<S : BaseFloat>(v : Vector2<S>, angle : S) -> cgmath::Vector2<S> {
    rotate_vector_ox(v, vec2(angle.cos(), angle.sin()))
}
   
/*
cgmath::vec2(
        Self::DIRECTION.y * direction.x + Self::DIRECTION.x * direction.y,
        -Self::DIRECTION.x * direction.x + Self::DIRECTION.y * direction.y,
    )
*/
