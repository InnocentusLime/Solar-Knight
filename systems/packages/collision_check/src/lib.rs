use nalgebra::{ Vector2, Isometry2 };
use parry2d::shape::{ Cuboid, Shape };
use parry2d::query::{ intersection_test, contact };
use parry2d::query::contact::Contact;
use parry2d::bounding_volume::AABB;

use ship_transform::Transform;
use systems_core::{ ComponentAccess, get_component };

#[derive(Clone, Copy, Debug)]
pub enum CollisionModelIndex {
    Player,
    EnemyTester,
    BulletTester,
    LaserBall,
    LaserBeam,
}

// TODO macro
struct ColliderTable {
    player : Cuboid,
    enemy_tester : Cuboid,
    bullet_tester : Cuboid,
    laser_ball : Cuboid,
    laser_beam : Cuboid,
}

impl ColliderTable {
    fn new() -> Self {
        ColliderTable {
            player : Cuboid::new(Vector2::new(0.1f32, 0.1f32)),
            enemy_tester : Cuboid::new(Vector2::new(0.1f32, 0.1f32)),
            bullet_tester : Cuboid::new(Vector2::new(0.09f32, 0.06f32)),
            laser_ball : Cuboid::new(Vector2::new(0.03f32, 0.03f32)),
            laser_beam : Cuboid::new(Vector2::new(sys_api::graphics_init::SCREEN_WIDTH / 1.5f32, 0.03f32)),
        }
    }

    fn decypher(&self, idx : CollisionModelIndex) -> &dyn Shape {
        use CollisionModelIndex::*;
        match idx {
            Player => &self.player,
            EnemyTester => &self.enemy_tester,
            BulletTester => &self.bullet_tester,
            LaserBall => &self.laser_ball,
            LaserBeam => &self.laser_beam,
        }
    }
}

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
}

pub struct CollisionSystem {
    colliders : ColliderTable,
}

impl CollisionSystem {
    pub fn new() -> Self {
        CollisionSystem {
            colliders : ColliderTable::new(),
        }
    }

    // TODO the `other` argument should be a another type (`Obj2`)
    // this is currently blocked purely because of the way `Bullets` are
    // implemented
    pub fn check<Obj>(&self, obj : &Obj, other_model : CollisionModelIndex, other_transform : &Isometry2<f32>) -> bool
    where
        Obj : ComponentAccess<Transform> + ComponentAccess<CollisionInfo>,
    {
        let model = get_component::<CollisionInfo, _>(obj).model;
        let transform = &get_component::<Transform, _>(obj).full_transform();

        let shape = self.colliders.decypher(model);
        let other_shape = self.colliders.decypher(other_model);

        intersection_test(transform, shape, other_transform, other_shape).unwrap()
    }

    // TODO the `other` argument should be a another type (`Obj2`)
    // this is currently blocked purely because of the way `Bullets` are
    // implemented
    pub fn check_contact<Obj>(
        &self, 
        obj : &Obj,
        other_model : CollisionModelIndex, other_transform : &Isometry2<f32>
    ) -> Option<Contact>
    where
        Obj : ComponentAccess<Transform> + ComponentAccess<CollisionInfo>,
    {
        const CONTACT_EPS : f32 = 0.00001f32;

        let model = get_component::<CollisionInfo, _>(obj).model;
        let transform = &get_component::<Transform, _>(obj).full_transform();

        let shape = self.colliders.decypher(model);
        let other_shape = self.colliders.decypher(other_model);

        contact(
            transform, shape, 
            other_transform, other_shape,
            CONTACT_EPS
        ).unwrap()
    }

    pub fn get_aabb<Obj>(&self, obj : &Obj) -> AABB
    where
        Obj : ComponentAccess<Transform> + ComponentAccess<CollisionInfo>,
    {
        let collision_info = get_component::<CollisionInfo, _>(obj).model;
        let transform = &get_component::<Transform, _>(obj).transform;
        self.colliders.decypher(collision_info).compute_aabb(transform)
    }
}
