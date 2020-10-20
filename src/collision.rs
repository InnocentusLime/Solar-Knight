use std::borrow::{ Borrow, BorrowMut };
use cgmath::{ SquareMatrix, EuclideanSpace, InnerSpace, Transform, Rad, Matrix4, Point2, Vector2, Vector4, vec2, vec4, dot };

fn max(x : f32, y : f32) -> f32 { x.max(y) }

fn min(x : f32, y : f32) -> f32 { x.min(y) }

static AXIS_ALIGNED_SQUARE : [Point2<f32>; 4] =
[
    Point2 { x : -1.0f32, y : -1.0f32 },
    Point2 { x : -1.0f32, y : 1.0f32 },
    Point2 { x : 1.0f32, y : 1.0f32 },
    Point2 { x : 1.0f32, y : -1.0f32 },
];

#[derive(Clone, Copy)]
pub struct AxisAlignedBoundingBox {
    pub left : f32,
    pub top : f32,
    pub right : f32,
    pub bottom : f32,
}

impl AxisAlignedBoundingBox {
    fn search_init() -> Self {
        AxisAlignedBoundingBox {
            left : std::f32::INFINITY,
            right : std::f32::NEG_INFINITY,
            top : std::f32::NEG_INFINITY,
            bottom : std::f32::INFINITY,
        }
    }

    pub fn of_mesh(mesh : &[Point2<f32>]) -> Self {
        mesh.iter()
        .fold(
            Self::search_init(),
            |acc, Point2 { x, y }| 
            AxisAlignedBoundingBox {
                left : min(acc.left, *x),
                right : max(acc.right, *x),
                top : max(acc.top, *y),
                bottom : min(acc.bottom, *y)
            }
        )
    }

    #[inline]
    pub fn of_circle(center : Point2<f32>, r : f32) -> Self {
        AxisAlignedBoundingBox {
            left : center.x - r,
            top : center.y + r,
            right : center.x + r,
            bottom : center.y + r,
        }
    }

    #[inline]
    pub fn of_two_aabb(self, other : Self) -> Self {
        AxisAlignedBoundingBox {
            left : min(self.left, other.left),
            top : max(self.top, other.top),
            right : max(self.right, other.right),
            bottom : min(self.bottom, other.bottom), 
        }
    }
   
    #[inline]
    pub fn collision_area(self, other : Self) -> Option<Self> {
        let left = max(self.left, other.left);
        let right = min(self.right, other.right);
        let top = min(self.top, other.top);
        let bottom = max(self.bottom, other.bottom);

        if left > right { return None }
        if bottom > top { return None }

        Some(
            AxisAlignedBoundingBox {
                left, top,
                right, bottom,
            }   
        )
    }

    #[inline]
    pub fn collision_test(self, other : Self) -> bool { self.collision_area(other).is_some() }
}

#[inline]
fn point_line_segment_distance(p : Point2<f32>, line_p_1 : Point2<f32>, line_p_2 : Point2<f32>) -> f32 {
    if dot(p - line_p_1, line_p_2 - line_p_1) < 0.0f32 {
        (p - line_p_1).magnitude()
    } else if dot(p - line_p_2, line_p_1 - line_p_2) < 0.0f32 {
        (p - line_p_2).magnitude()
    } else {
        let line_vec = line_p_2 - line_p_1;

        assert!(line_vec.x != 0.0f32); assert!(line_vec.y != 0.0f32);

        let h = vec2(-line_vec.y, line_vec.x);
        let c = -dot(h, line_p_1.to_vec());
        (dot(h, p.to_vec()) + c) / h.magnitude()
    }
}

#[derive(Clone, Copy)]
pub struct Circle {
    pub center : Point2<f32>,
    pub radius : f32,
}

#[derive(Clone, Copy)]
pub struct Mesh<M> {
    pub mem : M,
}

#[derive(Clone, Copy)]
pub struct Together<A, B> {
    pub a : A,
    pub b : B,
}

pub trait Collision<Other> {
    fn check(&self, other : &Other) -> bool;
}

impl Collision<Circle> for Circle {
    #[inline]
    fn check(&self, other : &Circle) -> bool {
        (self.center - other.center).magnitude() <= self.radius + other.radius
    }
}

impl<M> Collision<Mesh<M>> for Circle 
where
    M : Borrow<[Point2<f32>]>,
{
    fn check(&self, other : &Mesh<M>) -> bool {
        use std::iter::once;

        let mesh = other.mem.borrow();

        assert!(mesh.len() > 0);

        let mut sides = 
            mesh.windows(2).map(|p| (p[0], p[1]))
            .chain(once(
                (*mesh.last().unwrap(), *mesh.first().unwrap())
            ))
        ;

        sides.any(|(a, b)| point_line_segment_distance(self.center, a, b) <= self.radius)
    }
}

impl<A, B> Collision<Together<A, B>> for Circle 
where
    A : Collision<Circle>, 
    B : Collision<Circle>,
{
    #[inline]
    fn check(&self, other : &Together<A, B>) -> bool { other.a.check(self) || other.b.check(self) }
}

impl<A, B> Collision<Circle> for Together<A, B> 
where
    A : Collision<Circle>, 
    B : Collision<Circle>,
{
    #[inline]
    fn check(&self, other : &Circle) -> bool { self.a.check(other) || self.b.check(other) }
}

impl<M> Collision<Circle> for Mesh<M> 
where
    M : Borrow<[Point2<f32>]>,
{
    #[inline]
    fn check(&self, other : &Circle) -> bool { other.check(self) }
}

impl<M1, M2> Collision<Mesh<M2>> for Mesh<M1> 
where
    M1 : Borrow<[Point2<f32>]>, 
    M2 : Borrow<[Point2<f32>]>,
{
    fn check(&self, other : &Mesh<M2>) -> bool {
        // we use separating axis theorem here to check for collisions
        // https://en.wikipedia.org/wiki/Hyperplane_separation_theorem
        use std::iter::once;

        let a = self.mem.borrow();
        let b = other.mem.borrow();

        assert!(a.len() > 0);
        assert!(b.len() > 0);

        // iterators over sides of the meshes
        let a_sides = 
            a.windows(2).map(|p| (p[0], p[1]))
            .chain(once(
                (*a.last().unwrap(), *a.first().unwrap())
            ))
        ;
        let b_sides = 
            b.windows(2).map(|p| (p[0], p[1]))
            .chain(once(
                (*b.last().unwrap(), *b.first().unwrap())
            ))
        ;

        // iterator over all possible axis
        let mut axis = 
            a_sides.chain(b_sides)
            .map(
                |(a, b)| {
                    let v = a - b;
                    vec2(-v.y, v.x)
                }
            )
        ;

        // The aglrithm below checks for LACK of collisions (returns `true` when there's 
        // no collision and return `false` when there's a collision), so we
        // invert the result so the algorithm becomes an algrothm
        // whichs cheks for the PRESENCE of collisions (return `true` on collision and `false`
        // when there's no collision)
        !axis.any(
            |perp| {
                // projecting all points in search of the segment
                // which will represent the shape A
                let (a_min_proj, a_max_proj) =
                    a.iter()
                    .fold(
                        (std::f32::INFINITY, std::f32::NEG_INFINITY),
                        |(a_min_proj, a_max_proj), p| {
                            let x = dot(p.to_vec(), perp);
                            (min(a_min_proj, x), max(a_max_proj, x))
                        }
                    )
                ;

                // projecting all points in search of the segment
                // which will represent the shape B
                let (b_min_proj, b_max_proj) =
                    b.iter()
                    .fold(
                        (std::f32::INFINITY, std::f32::NEG_INFINITY),
                        |(b_min_proj, b_max_proj), p| {
                            let x = dot(p.to_vec(), perp);
                            (min(b_min_proj, x), max(b_max_proj, x))
                        }
                    )
                ;
    
                // check if we managed to split them
                // if we managed to split them, there was
                // no collision
                a_max_proj < b_min_proj || b_max_proj < a_min_proj
            }
        )
    }
}

impl<M, A, B> Collision<Together<A, B>> for Mesh<M> 
where
    M : Borrow<[Point2<f32>]>,
    A : Collision<Mesh<M>>, 
    B : Collision<Mesh<M>>,
{
    #[inline]
    fn check(&self, other : &Together<A, B>) -> bool { other.a.check(self) || other.b.check(self) }
}

impl<M, A, B> Collision<Mesh<M>> for Together<A, B> 
where
    M : Borrow<[Point2<f32>]>,
    A : Collision<Mesh<M>>, 
    B : Collision<Mesh<M>>,
{
    #[inline]
    fn check(&self, other : &Mesh<M>) -> bool { self.a.check(other) || self.b.check(other) }
}

impl<A1, B1, A2, B2> Collision<Together<A2, B2>> for Together<A1, B1> where
    A1 : Collision<A2> + Collision<B2>,
    A2 : Collision<A1> + Collision<B1>,
    B1 : Collision<A2> + Collision<B2>,
    B2 : Collision<A1> + Collision<B1>,
{
    #[inline]
    fn check(&self, other : &Together<A2, B2>) -> bool {
        other.a.check(&self.a) || other.a.check(&self.b) ||
        other.b.check(&self.a) || other.b.check(&self.b)
    }
}

pub trait Transformable {
    fn apply_transform<T>(self, trans : &T) -> Self
    where T : Transform<Point2<f32>>; 
}

impl Transformable for Circle {
    #[inline]
    fn apply_transform<T>(self, trans : &T) -> Self
    where T : Transform<Point2<f32>> { 
        let surface_point = self.center + vec2(self.radius, 0.0f32);

        let new_center = trans.transform_point(self.center);
        let new_surface_point = trans.transform_point(surface_point);

        Circle {
            center : new_center,
            radius : (new_surface_point - new_center).magnitude(),
        }
    }
}

impl<M> Transformable for Mesh<M> 
where
    M : BorrowMut<[Point2<f32>]>,
{
    #[inline]
    fn apply_transform<T>(mut self, trans : &T) -> Self
    where T : Transform<Point2<f32>> { 
        self.mem
        .borrow_mut()
        .iter_mut()
        .for_each(|p| *p = trans.transform_point(*p));

        self
    }
}

impl<A, B> Transformable for Together<A, B> 
where
    A : Transformable,
    B : Transformable,
{
    #[inline]
    fn apply_transform<T>(self, trans : &T) -> Self 
    where T : Transform<Point2<f32>> {
        Together {
            a : self.a.apply_transform(trans),
            b : self.b.apply_transform(trans),
        }
    }
}

#[macro_export]
macro_rules! body_expr {
    (
        (Circle : $center:expr, $radius:expr)
    ) => {
        Circle {
            center : Point2 { x : $center.0, y : $center.1 },
            radius : $radius,
        }
    };
    (
        (Mesh : $($p:expr),+)
    ) => {
        Mesh {
            mem : [$(Point2 { x : $p.0, y : $p.1 }),+],
        }
    };
    (
        [$first:tt]
    ) => {
        body_expr!($first)
    };
    (
        [$first:tt, $($elem:tt),+]
    ) => {
        Together {
            a : body_expr!($first),
            b : body_expr!([$($elem),+])
        }
    };
}

#[macro_export]
macro_rules! body_type {
    (
        (Circle : $center:expr, $radius:expr)
    ) => {
        Circle
    };
    (
        (Mesh : $($p:expr),+)
    ) => {
        Mesh<[Point2<f32>; [$($p),+].len()]>
    };
    (
        [$first:tt]
    ) => {
        body_type!($first)
    };
    (
        [$first:tt, $($elem:tt),+]
    ) => {
        Together<
            body_type!($first),
            body_type!([$($elem),+])
        >
    };
}

#[macro_export]
macro_rules! body_const {
    ($i:ident, $e:tt) => {
        pub const $i : body_type!($e) = body_expr!($e);
    };
}

#[macro_export]
macro_rules! declare_bodies {
    (
            $( $name:ident = $model:tt );+;
    ) => {
        pub mod bodies {
            use cgmath::Point2;
            use crate::body_type;
            use crate::collision::{ Circle, Mesh, Together };

            pub enum CollisionModel {
                $( $name (body_type!($model)) ),+
            }

            pub mod consts {
                use cgmath::Point2;
                use crate::{ body_type, body_expr };
                use crate::collision::{ Circle, Mesh, Together };

                use super::CollisionModel;

                $( 
                    #[allow(non_upper_case_globals)]
                    pub const $name : CollisionModel = CollisionModel::$name(body_expr!($model)); 
                )+
            }

            mod __collision_impls {
                use cgmath::{ Transform, Point2 };
                use std::borrow::{ Borrow, BorrowMut };
                use crate::collision::{ Circle, Mesh, Together, Collision, Transformable };

                use super::CollisionModel;

                impl Collision<Circle> for CollisionModel {
                    #[inline]
                    fn check(&self, other : &Circle) -> bool {
                        match self {
                            $(
                                CollisionModel::$name(x) => x.check(other)
                            ),+
                        }
                    }
                }
        
                impl<M> Collision<Mesh<M>> for CollisionModel 
                where
                    M : Borrow<[Point2<f32>]>,
                {
                    #[inline]
                    fn check(&self, other : &Mesh<M>) -> bool {
                        match self {
                            $(
                                CollisionModel::$name(x) => x.check(other)
                            ),+
                        }
                    }
                }
        
                impl<A, B> Collision<Together<A, B>> for CollisionModel 
                where
                    A : Collision<CollisionModel>,
                    B : Collision<CollisionModel>,
                {
                    #[inline]
                    fn check(&self, other : &Together<A, B>) -> bool {
                        other.a.check(self) || other.b.check(self)
                    }
                }

                impl Collision<CollisionModel> for Circle {
                    #[inline]
                    fn check(&self, other : &CollisionModel) -> bool {
                        other.check(self)
                    }
                }
        
                impl<M> Collision<CollisionModel> for Mesh<M>
                where
                    M : Borrow<[Point2<f32>]>,
                {
                    #[inline]
                    fn check(&self, other : &CollisionModel) -> bool {
                        other.check(self)
                    }
                }
        
                impl<A, B> Collision<CollisionModel> for Together<A, B>
                where
                    A : Collision<CollisionModel>,
                    B : Collision<CollisionModel>,
                {
                    #[inline]
                    fn check(&self, other : &CollisionModel) -> bool {
                        self.a.check(other) || self.b.check(other)
                    }
                }

                impl Collision<CollisionModel> for CollisionModel {
                    #[inline]
                    fn check(&self, other : &CollisionModel) -> bool {
                        match self {
                            $(
                                CollisionModel::$name(x) => other.check(x)
                            ),+
                        }
                    }
                }
        
                impl Transformable for CollisionModel {
                    fn apply_transform<T>(self, trans : &T) -> Self
                    where T : Transform<Point2<f32>> 
                    {
                        match self {
                            $(
                                CollisionModel::$name(x) => CollisionModel::$name(x.apply_transform(trans))
                            ),+
                        }
                    }
                }
            }
        }
    };
}
