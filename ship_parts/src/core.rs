use cgmath::assert_abs_diff_eq;
use cgmath::{ Point2, Vector2, Matrix3, EuclideanSpace, InnerSpace, vec2, point2 };

//use collision::*;
use crate::constants::VECTOR_NORMALIZATION_RANGE;
use crate::collision_models::{ CollisionModel, model_indices };
use model_indices::*;
use cgmath_ext::matrix3_from_translation;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Team {
    Hive,
    Earth,
}

/// # Description
/// The core of the ship. 
/// This struct contains commons data that is owned
/// by every ship. This includes:
/// - Hitpoints
/// - Collision model index
/// - Team id
/// - Mass
/// - Position
/// - Direction
/// - Current force
/// - Velocity
///
/// # Invariants
/// There are some invariants that the inner data must satisfy
/// - Mass must be positive
/// - Mass, pos, direction, force, velocity must be all finite vectors
///
/// # Technical details
/// It is absolutely fine for a ship to have 0hp. This state might
/// be used to do some behaviour to emulate a death state for a boss.
// TODO consider inlining `Core` into `Ship`
// FIXME `mass`, `force`, `velocity` and `direction`, `pos` should be protected
#[derive(Clone, Copy)]
pub struct Core {
    hp : u64,
    model : CollisionModelIndex,
    team : Team,
    pub mass : f32,
    pub pos : Point2<f32>,
    direction : Vector2<f32>,
    pub force : Vector2<f32>,
    pub velocity : Vector2<f32>,
}

impl Core {
    pub const fn new(hp : u64, mass : f32, model : CollisionModelIndex, team : Team) -> Self {
        Core {
            hp,
            model,
            team,
            pos : point2(0.0f32, 0.0f32),
            mass,
            direction : vec2(0.0f32, 1.0f32),
            force : vec2(0.0f32, 0.0f32),
            velocity : vec2(0.0f32, 0.0f32),
        }
    }

    #[inline]
    pub fn team(&self) -> Team { self.team }

    #[inline]
    pub fn model(&self) -> CollisionModelIndex { self.model }

    #[inline]
    pub fn hp(&self) -> u64 {
        self.hp
    }

    /// Call only when absolutely needed.
    ///
    /// Unsafety
    /// Allows you to set HP avoiding any damage triggers
    #[inline]
    pub unsafe fn hp_mut(&mut self) -> &mut u64 {
        &mut self.hp
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    #[inline]
    pub fn damage(&mut self, dmg : u64) {
        self.hp = self.hp.saturating_sub(dmg);
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

    pub fn phys_body(&self) -> CollisionModel {
        use collision::*;

        self.model.decypher()
        .apply_transform(&self.transform())
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
}
