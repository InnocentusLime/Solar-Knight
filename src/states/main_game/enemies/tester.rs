use cgmath::{ Angle, Matrix4, EuclideanSpace, InnerSpace, Rad, Point2, Decomposed, Vector2, Basis2, Rotation2, vec2 };

use crate::collision::Transformable;
use crate::collision_models;
use collision_models::CollisionModel;
use crate::states::main_game::player::Player;

pub const TESTER_SIZE : (f32, f32) = (0.1f32, 0.1f32);
pub const TESTER_SPAWN_HP : u64 = 10;
pub const TESTER_STEP_LENGTH : f32 = 0.001f32;

pub struct Tester {
    pub pos : Point2<f32>,
    pub ang : Rad<f32>,
    pub hp : u64,
}

impl Tester {
    pub fn new() -> Self {
        Tester {
            pos : Point2 { x : 0.0f32, y : 0.0f32 },
            ang : Rad(0.0f32),
            hp : TESTER_SPAWN_HP,
        }
    }

    pub fn update(&mut self, player : &Player) {
        let player_pos = player.pos;
        let ang = vec2(0.0f32, 1.0f32).angle(player_pos.to_vec() - self.pos.to_vec());
        let (s, c) = ang.sin_cos();

        if self.hp > 0 {
            self.ang = ang; 
            self.pos += vec2(-s, c) * TESTER_STEP_LENGTH;
        } else {
            // we turn into a spiny boi on death
            self.ang += Rad(std::f32::consts::TAU / 360.0f32 * 4.0f32);
        }
    }
    
    #[inline]
    pub fn transform(&self) -> Decomposed<Vector2<f32>, Basis2<f32>> {
        Decomposed {
            scale : 1.0f32,
            rot : <Basis2<f32> as Rotation2<f32>>::from_angle(self.ang),
            disp : self.pos.to_vec(),
        }
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_angle_z(self.ang) * Matrix4::from_nonuniform_scale(TESTER_SIZE.0, TESTER_SIZE.1, 1.0f32)

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
