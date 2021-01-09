use std::time::Duration;

use glium::VertexBuffer;
use cgmath::{ InnerSpace, EuclideanSpace, Matrix4, Angle, Rad, Point2, Vector2, Matrix3, Rotation2, Basis2, vec2 };

use super::enemies::{ Hive, Enemy };
use crate::graphics_init::ENEMY_BULLET_LIMIT;
use crate::basic_graphics_data::SpriteData;
use crate::containers::MemoryChunk;
use crate::collision_models;
use crate::collision::Collision;
use crate::transform2d_utils::*;
use crate::duration_ext::*;

const PLAYER_SIZE : (f32, f32) = (0.1f32, 0.1f32);
const TESTER_BULLET_SIZE : (f32, f32) = (0.06f32, 0.09f32);
const PLAYER_SPEED : f32 = 0.21f32;

const PLAYER_MAX_SPEED : u64 = 4;
const PLAYER_BULLET_SPEED : f32 = 4.0f32;
const PLAYER_BULLET_LIFE_LENG : Duration = Duration::from_secs(3);
const PLAYER_BULLET_RECOIL : Duration = Duration::from_millis(133);

const PLAYER_DASH_TRACE_SPEED : f32 = 0.6f32;
const PLAYER_DASH_TRACE_LIFE_LENG : Duration = Duration::from_millis(218);
const PLAYER_DASH_LIFE_LENG : Duration = Duration::from_millis(150);
const PLAYER_DASH_DISTANCE : f32 = 0.55f32;
const PLAYER_DASH_FRAK : f32 = std::f32::consts::E;
const PLAYER_SPEED_IGNORE_STRENGTH : f32 = 8.0f32;

fn player_dash_func(x : f32) -> f32 {
    PLAYER_DASH_FRAK.powf(PLAYER_DASH_LIFE_LENG.as_secs_f32() - x + 1.0f32) - PLAYER_DASH_FRAK
}

// integral from 0 to x of player_dash_func
fn player_dash_func_integral(x : f32) -> f32 {
    -(PLAYER_DASH_FRAK.powf(PLAYER_DASH_LIFE_LENG.as_secs_f32() - x + 1.0f32) / PLAYER_DASH_FRAK.ln() + PLAYER_DASH_FRAK * x)
}

pub struct TestBullet {
    direction : Vector2<f32>,     // Flight direction 
    pos : Point2<f32>,  // The buller position
    lifetime : Duration,     // The remaining frames for the bullet to live
}

impl TestBullet {
    pub fn new(direction : Vector2<f32>, pos : Point2<f32>) -> Self {
        TestBullet {
            direction, pos,
            lifetime : PLAYER_BULLET_LIFE_LENG,
        }
    }

    pub fn update(&mut self, hive : &mut Hive, dt : Duration) {
        use crate::collision::*;

        self.pos += dt.as_secs_f32() * PLAYER_BULLET_SPEED * self.direction;
        self.lifetime = self.lifetime.my_saturating_sub(dt);

        let my_body = collision_models::consts::BulletTester.apply_transform(&self.transform());
        let my_aabb = my_body.aabb();

        for enemy in hive.iter_mut() {
            if self.lifetime.my_is_zero() { break }

            let enemy_body = enemy.phys_body();
            let enemy_aabb = enemy_body.aabb();
            
            if *enemy.hp() > 0 && enemy_aabb.collision_test(my_aabb) && enemy_body.check_collision(&my_body) {
                *enemy.hp_mut() -= 1;
                self.lifetime = <Duration as DurationExt>::my_zero();
            } 
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
        Matrix4::from_nonuniform_scale(TESTER_BULLET_SIZE.0, TESTER_BULLET_SIZE.1, 1.0f32)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DashState {
    Performing {
        trace_lifetime : Duration,
        lifetime : Duration,
        direction : Vector2<f32>,
        trace_pos : Point2<f32>,
        trace_direction : Vector2<f32>,
    },
    Done,
}

pub struct Player {
    direction : Vector2<f32>,
    pos : Point2<f32>,
    bullets : MemoryChunk<TestBullet>,
    recoil : Duration,
    speed : u64,
    dash_info : DashState,
}

impl Player {
    pub fn new() -> Self {
        Player {
            recoil : <Duration as DurationExt>::my_zero(),
            direction : vec2(0.0f32, 0.0f32),
            pos : Point2 { x : 0.0f32, y : 0.0f32 },
            bullets : MemoryChunk::with_capacity(ENEMY_BULLET_LIMIT),
            speed : 0,
            dash_info : DashState::Done,
        }
    }

    #[inline]
    pub fn pos(&self) -> Point2<f32> { self.pos }

    pub fn dash_right(&mut self) {
        match (*self).dash_info {
            DashState::Performing { .. } => (),
            DashState::Done => { 
                let player_push = (self.speed as f32 / PLAYER_SPEED_IGNORE_STRENGTH) * self.direction;    
                let dash_direction = vec2(self.direction.y, -self.direction.x);

                self.dash_info = 
                    DashState::Performing {
                        trace_lifetime : PLAYER_DASH_TRACE_LIFE_LENG,
                        lifetime : PLAYER_DASH_LIFE_LENG, 
                        direction : dash_direction, 
                        trace_pos : self.pos,
                        trace_direction : (-(player_push + dash_direction)).normalize(),
                    }
                ;
            },
        }
    }
    
    pub fn dash_left(&mut self) {
        match (*self).dash_info {
            DashState::Performing { .. } => (),
            DashState::Done => {
                let player_push = (self.speed as f32 / PLAYER_SPEED_IGNORE_STRENGTH) * self.direction;    
                let dash_direction = vec2(-self.direction.y, self.direction.x);

                self.dash_info = 
                    DashState::Performing { 
                        trace_lifetime : PLAYER_DASH_TRACE_LIFE_LENG,
                        lifetime : PLAYER_DASH_LIFE_LENG, 
                        direction : dash_direction, 
                        trace_pos : self.pos,
                        trace_direction : (-(player_push + dash_direction)).normalize(),
                    }
                ;
            },
        }
    }

    #[inline]
    pub fn increase_speed(&mut self) {
        self.speed = PLAYER_MAX_SPEED.min(self.speed + 1);
    }

    #[inline]
    pub fn decrease_speed(&mut self) {
        self.speed = self.speed.saturating_sub(1);
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

    #[inline]
    pub fn dash_trace_model_mat(&self) -> Option<Matrix4<f32>> {
        match self.dash_info {
            DashState::Performing {
                trace_lifetime,   
                trace_direction,
                trace_pos,
                ..
            } if !trace_lifetime.my_is_zero() => {
                let k = 2.0f32 * (trace_lifetime.as_secs_f32()) / (PLAYER_DASH_TRACE_LIFE_LENG.as_secs_f32());
                let model_mat = 
                    Matrix4::from_translation(trace_pos.to_vec().extend(0.0f32)) * 
                    Matrix4::new(
                        trace_direction.x, trace_direction.y, 0.0f32, 0.0f32,
                        -trace_direction.y, trace_direction.x, 0.0f32, 0.0f32,
                        0.0f32, 0.0f32, 1.0f32, 0.0f32,
                        0.0f32, 0.0f32, 0.0f32, 1.0f32,
                    ) * 
                    Matrix4::from_nonuniform_scale(k * 0.04f32, (k / 2.0f32 * 0.4f32 + 0.6f32) * 0.125f32, 1.0f32) 
                ;
                Some(model_mat)
            },
            _ => None,
        }
    }

    pub fn update(&mut self, direction : Vector2<f32>, dt : Duration) {
        self.recoil = self.recoil.my_saturating_sub(dt);
        self.direction = direction.normalize();

        self.pos += (self.speed as f32) * 0.01f32 * self.direction;     
       
        let mut dash_info = self.dash_info;
        match &mut dash_info {
            DashState::Performing {
                trace_lifetime,
                lifetime,
                direction,
                trace_pos,
                trace_direction,
                ..
            } => {
                let sum : f32 = player_dash_func_integral(PLAYER_DASH_LIFE_LENG.as_secs_f32()) - player_dash_func_integral(0.0f32);
                let frak = (player_dash_func(PLAYER_DASH_LIFE_LENG.as_secs_f32() - lifetime.as_secs_f32()) / sum) * dt.as_secs_f32();
                let speed = frak * PLAYER_DASH_DISTANCE;

                self.pos += speed * (*direction);
                if !trace_lifetime.my_is_zero() {                
                    *trace_lifetime = trace_lifetime.my_saturating_sub(dt);
                    *trace_pos += PLAYER_DASH_TRACE_SPEED * dt.as_secs_f32() * (*trace_direction);
                }

                *lifetime = lifetime.my_saturating_sub(dt); 
                if lifetime.my_is_zero() { 
                    self.dash_info = DashState::Done 
                } else {
                    self.dash_info = dash_info
                }
            },
            DashState::Done => (),
        }
    }

    pub fn update_bullets(&mut self, hive : &mut Hive, dt : Duration) {
        self.bullets.iter_mut()
        .for_each(|x| x.update(hive, dt));

        self.bullets.retain(|x| !x.lifetime.my_is_zero());
    }

    pub fn shoot(&mut self) {
        if self.recoil.my_is_zero() {
            self.recoil = PLAYER_BULLET_RECOIL;
            self.bullets.push(TestBullet::new(self.direction, self.pos));
        }
    }

    pub fn fill_bullet_buffer(&self, buff : &mut VertexBuffer<SpriteData>) {
        let mut ptr = buff.map_write();

        if ptr.len() < ENEMY_BULLET_LIMIT { panic!("Buffer too small"); }

        for i in 0..ptr.len() { 
            use crate::basic_graphics_data::ZEROED_SPRITE_DATA;
            
            ptr.set(i, ZEROED_SPRITE_DATA);
        }

        self.bullets.iter()
        .enumerate()
        .for_each(|(i, x)| {
            let m = x.model_mat();
            
            let dat =
                SpriteData {
                    mat_col1 : m.x.into(),
                    mat_col2 : m.y.into(),
                    mat_col3 : m.z.into(),
                    mat_col4 : m.w.into(),
                    texture_bottom_left : [0.0f32, 0.0f32],
                    texture_top_right : [1.0f32, 1.0f32],
                }
            ;
            
            ptr.set(i, dat);
        });
    }
}
