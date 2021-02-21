use std::time::Duration;

use glium::VertexBuffer;
use cgmath::{ InnerSpace, EuclideanSpace, Matrix4, Angle, Rad, Point2, Vector2, Matrix3, Rotation2, Basis2, vec2 };

//use super::enemies::{ Hive, Enemy };
use super::ship_parts::gun::{ Bullet, BulletKind };
use crate::graphics_init::ENEMY_BULLET_LIMIT;
use crate::basic_graphics_data::SpriteData;
use crate::containers::MemoryChunk;
use crate::collision_models;
use crate::collision::Collision;
use crate::transform2d_utils::*;
use crate::duration_ext::*;
use crate::{ exponential_decrease_curve, declare_engine, declare_gun };

/*
const PLAYER_SIZE : (f32, f32) = (0.1f32, 0.1f32);

const PLAYER_MAX_SPEED : u64 = 4;

const PLAYER_DASH_LIFE_LENG : Duration = Duration::from_millis(200);
const PLAYER_DASH_FRAK : f32 = std::f32::consts::E;

const PLAYER_DASH_SPEED : f32 = 8.0f32;


/*
declare_gun!(
    inf_gun TestGun {
        offset : vec2(0.0f32, 0.0f32),
        bullet_kind : tester_bullet,
        recoil : Duration::from_millis(133),
        direction : vec2(0.0f32, 1.0f32),
    }
);
*/

pub struct Player {
    direction : Vector2<f32>,
    pos : Point2<f32>,

    engine : Engine,
    ldash : LDash,
    rdash : RDash,
    //gun : TestGun,
}

impl Player {
    pub const MAX_SPEED : u64 = Engine::MAX_LVL;

    pub fn new() -> Self {
        Player {
            direction : vec2(0.0f32, 0.0f32),
            pos : Point2 { x : 0.0f32, y : 0.0f32 },

            engine : Engine::new(0),
            ldash : LDash::new(0),
            rdash : RDash::new(0),
            //gun : TestGun::new(),
        }
    }

    #[inline]
    pub fn pos(&self) -> Point2<f32> { self.pos }

    #[inline]
    pub fn direction(&self) -> Vector2<f32> { self.direction }

    #[inline]
    pub fn is_dashing(&self) -> bool {
        self.ldash.is_changing() || self.rdash.is_changing()
    }

    pub fn dash_right(&mut self) -> Option<Vector2<f32>> {
        if !self.is_dashing() {
            // Check `dash_left` for info about how it works
            self.rdash.snap(1);
            self.rdash.decrease_speed();
            // return the dash direction
            Some(self.rdash.map_direction(self.direction))
        } else { None }
    }
    
    pub fn dash_left(&mut self) -> Option<Vector2<f32>> {
        if !self.is_dashing() {
            // This code will make the engine
            // update all its interior data to be `1`.
            // We'll then call `decrease_speed` to make engine's
            // speed decrease from 1 to 0.
            self.ldash.snap(1);
            self.ldash.decrease_speed();
            // return the dash direction
            Some(self.ldash.map_direction(self.direction))
        } else { None }
    }

    #[inline]
    pub fn increase_speed(&mut self) {
        self.engine.increase_speed();
    }

    #[inline]
    pub fn decrease_speed(&mut self) {
        self.engine.decrease_speed();
    }

    #[inline]
    pub fn get_speed(&self) -> u64 {
        self.engine.get_speed()
    }

    #[inline]
    pub fn point_at(&self, at : Point2<f32>) -> Option<Point2<f32>> {
        use crate::graphics_init::SCREEN_WIDTH;

        let v = at - self.pos;
        let x = (-SCREEN_WIDTH).max(SCREEN_WIDTH.min(v.x / v.y.abs()));
        let y = (-1.0f32).max(1.0f32.min(SCREEN_WIDTH * v.y / v.x.abs()));
        let pointer_v = vec2(x, y);

        if pointer_v.magnitude2() > v.magnitude2() { None }
        else { Some(<Point2<f32> as EuclideanSpace>::from_vec(pointer_v)) }
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
        Matrix4::from_nonuniform_scale(PLAYER_SIZE.0, PLAYER_SIZE.1, 1.0f32)
    }

    // Returns parameter `t` which encodes the progress of the
    // dash. `t` is stuffed into the [0; 1] range, where 1 denotes
    // the start of the dash and 0 denotes the end.
    #[inline]
    pub fn dash_trace_param(&self) -> Option<f32> {
        if self.is_dashing() {
            Some(self.ldash.get_speed() + self.rdash.get_speed())
        } else { 
            None 
        }
    }

    pub fn update(&mut self, direction : Vector2<f32>, dt : Duration) {
        self.direction = direction.normalize();
      
        self.pos = 
            self.rdash.update(
                self.ldash.update(
                    self.engine.update(
                        self.pos, 
                        self.direction, 
                        dt
                    ), 
                    self.direction, 
                    dt
                ),
                self.direction,
                dt
            )
        ;
    }

    pub fn shoot(&mut self) -> Option<Bullet> {
        //self.gun.shoot(self.pos, self.direction)
        unimplemented!()
    }
}

*/
