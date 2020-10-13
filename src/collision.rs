use cgmath::{ EuclideanSpace, Rad, Matrix4, Point2, Vector2, Vector4, vec2, vec4, dot };

use crate::basic_graphics_data::QUAD_VERTEX_DATA;

fn max(x : f32, y : f32) -> f32 { x.max(y) }

fn min(x : f32, y : f32) -> f32 { x.min(y) }

static AXIS_ALIGNED_SQUARE : [Point2<f32>; 4] =
[
    Point2 { x : -1.0f32, y : -1.0f32 },
    Point2 { x : -1.0f32, y : 1.0f32 },
    Point2 { x : 1.0f32, y : 1.0f32 },
    Point2 { x : 1.0f32, y : -1.0f32 },
];

#[inline]
pub fn mesh_of_sprite(model_mat : Matrix4<f32>, size : Vector2<f32>) -> [Point2<f32>; 4] {
    let mut output = [Point2 { x : 0.0f32, y : 0.0f32 }; 4];

    AXIS_ALIGNED_SQUARE.iter()
    .map(|v| model_mat * vec4(v.x * size.x, v.y * size.y, 0.0f32, 1.0f32))
    .map(|v| Point2 { x : v.x, y : v.y })
    .enumerate()
    .for_each(|(i, x)| output[i] = x);

    //println!("{:?}", output);

    output
}

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

    /// Axis Aligned Bounding Box
    #[inline]
    pub fn of_sprite(model_mat : Matrix4<f32>, size : Vector2<f32>) -> Self {
        AxisAlignedBoundingBox::of_mesh(&mesh_of_sprite(model_mat, size))
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
    
    /// Bounding box of a body. 
    pub fn of_body(body : &Body) -> AxisAlignedBoundingBox {
        // Zero allocation impl. So proud UwU
        match body {
            Body::Circle { center, r } => Self::of_circle(*center, *r),
            Body::Mesh { mesh } => Self::of_mesh(&mesh),
            Body::BodySystem { bodies } => {
                let mut iter = 
                    bodies.iter()
                    .map(|x| Self::of_body(x))
                ;

                let first = iter.next().expect("Body system can't be empty");

                iter.fold(first, Self::of_two_aabb)
            },
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

pub fn mesh_collision(a : &[Point2<f32>], b : &[Point2<f32>]) -> bool {
    // we use separating axis theorem here to check for collisions
    // https://en.wikipedia.org/wiki/Hyperplane_separation_theorem
    use std::iter::once;
    use std::borrow::Borrow;

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

#[inline]
fn point_line_distance(p : Point2<f32>, line_p_1 : Point2<f32>, line_p_2 : Point2<f32>) -> f32 {
    let v = line_p_1 - line_p_2;
    let a = p.x - v.y;
    let b = p.y + v.x;
    f32::hypot(a, b)
}

#[inline]
pub fn mesh_circle_collision(mesh : &[Point2<f32>], center : Point2<f32>, r : f32) -> bool {
    use std::iter::once;
    use std::borrow::Borrow;

    let mut sides = 
        mesh.windows(2).map(|p| (p[0], p[1]))
        .chain(once(
            (*mesh.last().unwrap(), *mesh.first().unwrap())
        ))
    ;

    sides.any(|(a, b)| point_line_distance(a, b, center) <= r)
}

#[inline]
pub fn circle_collision(center1 : Point2<f32>, r1 : f32, center2 : Point2<f32>, r2 : f32) -> bool {
    f32::hypot(center1.x - center2.x, center1.y - center2.y) <= r1 + r2
}

pub enum Body {
    Circle {
        center : Point2<f32>,
        r : f32,
    },
    Mesh {
        mesh : Vec<Point2<f32>>,
    },
    BodySystem {
        bodies : Vec<Body>,
    }
}

impl Body {
    pub fn collision_body_circle(&self, center : Point2<f32>, r : f32) -> bool {
        match self {
            Body::Circle { center : center0, r : r0 } => circle_collision(*center0, *r0, center, r),
            Body::Mesh { mesh } => mesh_circle_collision(mesh, center, r),
            Body::BodySystem { bodies } => bodies.iter().any(|body| body.collision_body_circle(center, r)),
        }
    }

    pub fn collision_body_mesh(&self, mesh : &[Point2<f32>]) -> bool {
        match self {
            Body::Circle { center, r } => mesh_circle_collision(mesh, *center, *r),
            Body::Mesh { mesh : mesh0 } => mesh_collision(mesh, mesh0),
            Body::BodySystem { bodies } => bodies.iter().any(|body| body.collision_body_mesh(mesh)), 
        }
    }

    #[inline]
    pub fn collision_body_body_system(&self, bodies : &[Body]) -> bool {
        bodies.iter().any(|body| self.collision_body_body(body))
    }

    #[inline]
    pub fn collision_body_body(&self, other : &Body) -> bool {
        match other {
            Body::Circle { center, r } => self.collision_body_circle(*center, *r),
            Body::Mesh { mesh } => self.collision_body_mesh(mesh),
            Body::BodySystem { bodies } => self.collision_body_body_system(bodies),
        }
    }

}
