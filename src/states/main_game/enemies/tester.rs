use std::time::Duration;

use cgmath::{ Angle, Matrix4, EuclideanSpace, InnerSpace, Rad, Point2, Matrix3, Vector2, Basis2, Rotation2, vec2 };

use crate::collision::Transformable;
use crate::collision_models;
use collision_models::CollisionModel;
use crate::states::main_game::player::Player;
use crate::transform2d_utils::*;

pub const TESTER_SIZE : (f32, f32) = (0.1f32, 0.1f32);
pub const TESTER_SPAWN_HP : u64 = 10;
pub const TESTER_SPEED : f32 = 0.06f32;

pub struct Tester {
    pos : Point2<f32>,
    direction : Vector2<f32>,
    hp : u64,
}

impl Tester {
    pub fn new() -> Self {
        Tester {
            pos : Point2 { x : 0.0f32, y : 0.0f32 },
            direction : vec2(0.0f32, 0.0f32),
            hp : TESTER_SPAWN_HP,
        }
    }

    pub fn update(&mut self, player : &Player, dt : Duration) {
        let player_pos = player.pos();
        let direction = (player_pos.to_vec() - self.pos.to_vec()).normalize();

        if self.hp > 0 {
            self.direction = direction;
            self.pos += (TESTER_SPEED * dt.as_secs_f32()) * self.direction;
        } else {
            // we turn into a spiny boi on death
            
            //self.ang += Rad(std::f32::consts::TAU / 360.0f32 * 4.0f32);
        }
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

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * 
        Matrix4::new(
            self.direction.y, -self.direction.x, 0.0f32, 0.0f32,
            self.direction.x, self.direction.y, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ) * 
        Matrix4::from_nonuniform_scale(TESTER_SIZE.0, TESTER_SIZE.1, 1.0f32)
    }

    #[inline]
    pub fn hp(&self) -> &u64 { &self.hp }

    #[inline]
    pub fn hp_mut(&mut self) -> &mut u64 { &mut self.hp }

    #[inline]
    pub fn collision_model(&self) -> CollisionModel {
        collision_models::consts::EnemyTester.apply_transform(&self.transform()) 
    }
}
