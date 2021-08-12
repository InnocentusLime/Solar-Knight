use cgmath::{ Matrix3, EuclideanSpace };

use collision::{ Collision, declare_bodies };
use cgmath_ext::matrix3_from_translation;

use ship_transform::Transform;
use systems_core::{ ComponentAccess, get_component };

// TODO laser ball is a circle!!!
// TODO `declare_bodies` could use a `box` variant
declare_bodies!(
    Player = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 0.1f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    EnemyTester = (Mesh : (-0.1f32, 0.1f32), (0.1f32, 0.1f32), (0.1f32, -0.1f32), (-0.1f32, -0.1f32));
    BulletTester = (Mesh : (-0.06f32, 0.09f32), (0.06f32, 0.09f32), (0.06f32, -0.09f32), (-0.06f32, -0.09f32));
    LaserBall = (Mesh : (-0.03f32, 0.03f32), (0.03f32, 0.03f32), (0.03f32, -0.03f32), (-0.03f32, -0.03f32));
    LaserBeam = (Mesh : (-0.03f32, sys_api::graphics_init::SCREEN_WIDTH / 1.5f32), (0.03f32, sys_api::graphics_init::SCREEN_WIDTH / 1.5f32), (0.03f32, 0.0f32), (-0.03f32, 0.0f32));
);

pub use bodies::*;
pub use model_indices::*;

#[derive(Clone, Copy, Debug)]
pub struct CollisionInfo {
    // Public for now since we don't do any
    // crazy stuff within the collision system
    pub model : CollisionModelIndex,
}

impl CollisionInfo {
    pub const fn new(model : CollisionModelIndex) -> Self {
        CollisionInfo {
            model,
        }
    }
    
    #[inline]
    pub fn model(&self) -> CollisionModelIndex { self.model }
    
    #[inline]
    fn transform(&self, transform : &Transform) -> Matrix3<f32> {
        matrix3_from_translation(transform.pos.to_vec()) *
        Matrix3::new(
            transform.direction().y, -transform.direction().x, 0.0f32,
            transform.direction().x, transform.direction().y, 0.0f32,
            0.0f32, 0.0f32, 1.0f32,
        )
    }

    fn phys_body(&self, transform : &Transform) -> CollisionModel {
        use collision::*;

        self.model.decypher()
        .apply_transform(&self.transform(transform))
    }
}

pub struct CollisionSystem {

}

impl CollisionSystem {
    pub fn new() -> Self {
        CollisionSystem {}
    }

    // TODO the `other` argument should be a another type (`Obj2`)
    // this is currently blocked purely because of the way `Bullets` are
    // implemented
    pub fn check<Obj>(&self, obj : &Obj, other : &CollisionModel) -> bool
    where
        Obj : ComponentAccess<Transform> + ComponentAccess<CollisionInfo>,
    {
        let collision_info = get_component::<CollisionInfo, _>(obj);
        let transform = get_component::<Transform, _>(obj);
        collision_info.phys_body(transform).check_collision(other)
    }
}
