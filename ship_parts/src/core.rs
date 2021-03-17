use cgmath::{ Point2, Vector2, Matrix3, Matrix4, EuclideanSpace, vec2 };

//use collision::*;
use crate::collision_models::{ CollisionModel, model_indices };
use model_indices::*;
use cgmath_ext::matrix3_from_translation;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Team {
    Hive,
    Earth,
}

pub struct Core {
    hp : u64,
    model : CollisionModelIndex,
    team : Team,
    pub mass : f32,
    pub pos : Point2<f32>,
    pub direction : Vector2<f32>,
    pub force : Vector2<f32>,
    pub velocity : Vector2<f32>,
}

impl Core {
    pub fn new(hp : u64, mass : f32, model : CollisionModelIndex, team : Team, pos : Point2<f32>, direction : Vector2<f32>) -> Self {
        Core {
            hp,
            model,
            team,
            pos,
            mass,
            direction,
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
}
