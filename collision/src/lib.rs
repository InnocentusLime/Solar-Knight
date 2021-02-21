pub mod base;

pub use base::*;

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
            use $crate::body_type;
            use $crate::base::*;

            #[derive(Clone, Copy, Debug)]
            pub enum CollisionModel {
                $( $name (body_type!($model)) ),+
            }

            pub mod consts {
                use cgmath::Point2;
                use $crate::base::*;
                use $crate::{ body_type, body_expr };

                use super::CollisionModel;

                $( 
                    #[allow(non_upper_case_globals)]
                    pub const $name : CollisionModel = CollisionModel::$name(body_expr!($model)); 
                )+
            }

            pub mod model_indices {
                use super::{ CollisionModel, consts };

                #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
                pub enum CollisionModelIndex {
                    $(
                        $name,
                    )+
                }

                impl CollisionModelIndex {
                    pub fn decypher(self) -> CollisionModel {
                        match self {
                            $( CollisionModelIndex::$name => consts::$name, )+
                        }
                    }
                }
            }

            mod __collision_impls {
                use cgmath::{ Transform, Point2 };
                use std::borrow::{ Borrow, BorrowMut };
                use $crate::base::*;

                use super::CollisionModel;

                impl Collision<Circle> for CollisionModel {
                    #[inline]
                    fn check_collision(&self, other : &Circle) -> bool {
                        match self {
                            $(
                                CollisionModel::$name(x) => x.check_collision(other)
                            ),+
                        }
                    }
                }
        
                impl<M> Collision<Mesh<M>> for CollisionModel 
                where
                    M : Borrow<[Point2<f32>]>,
                {
                    #[inline]
                    fn check_collision(&self, other : &Mesh<M>) -> bool {
                        match self {
                            $(
                                CollisionModel::$name(x) => x.check_collision(other)
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
                    fn check_collision(&self, other : &Together<A, B>) -> bool {
                        other.a.check_collision(self) || other.b.check_collision(self)
                    }
                }

                impl Collision<CollisionModel> for Circle {
                    #[inline]
                    fn check_collision(&self, other : &CollisionModel) -> bool {
                        other.check_collision(self)
                    }
                }
        
                impl<M> Collision<CollisionModel> for Mesh<M>
                where
                    M : Borrow<[Point2<f32>]>,
                {
                    #[inline]
                    fn check_collision(&self, other : &CollisionModel) -> bool {
                        other.check_collision(self)
                    }
                }
        
                impl<A, B> Collision<CollisionModel> for Together<A, B>
                where
                    A : Collision<CollisionModel>,
                    B : Collision<CollisionModel>,
                {
                    #[inline]
                    fn check_collision(&self, other : &CollisionModel) -> bool {
                        self.a.check_collision(other) || self.b.check_collision(other)
                    }
                }

                impl Collision<CollisionModel> for CollisionModel {
                    #[inline]
                    fn check_collision(&self, other : &CollisionModel) -> bool {
                        match self {
                            $(
                                CollisionModel::$name(x) => other.check_collision(x)
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

                impl ComputeAxisAlignedBoundingBox for CollisionModel {
                    fn aabb(&self) -> AxisAlignedBoundingBox {
                        match self {
                            $(
                                CollisionModel::$name(x) => x.aabb()
                            ),+
                        }
                    }
                }
            }
        }
    };
}

/*
declare_bodies!(
    Player = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 01f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    EnemyTester = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 0.1f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    BulletTester = (Mesh : (-0.06f32, 0.09f32), (0.06f32, 0.09f32), (0.06f32, -0.09f32), (-0.06f32, -0.09f32));
);
*/
