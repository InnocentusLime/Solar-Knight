use glium::VertexBuffer;
use cgmath::{ InnerSpace, EuclideanSpace, Matrix4, Angle, Rad, Point2, Vector2, Decomposed, Rotation2, Basis2, vec2 };

use super::enemies::{ Hive, Enemy };
use crate::graphics_init::ENEMY_BULLET_LIMIT;
use crate::basic_graphics_data::SpriteData;
use crate::containers::MemoryChunk;
use crate::collision_models;
use crate::collision::Collision;

const PLAYER_SIZE : (f32, f32) = (0.1f32, 0.1f32);
const TESTER_BULLET_SIZE : (f32, f32) = (0.06f32, 0.09f32);
const PLAYER_STEP_LENGTH : f32 = 0.007f32;

const PLAYER_MAX_SPEED : u64 = 5;
const PLAYER_BULLET_STEP_LENGTH : f32 = 0.05f32;
const PLAYER_BULLET_LIFE_LENG : u64 = 300;
const PLAYER_BULLET_RECOIL : u64 = 15;
pub struct TestBullet {
    pub ang : Rad<f32>,     // Flight direction 
    pub pos : Point2<f32>,  // The buller position
    pub lifetime : u64,     // The remaining frames for the bullet to live
}

impl TestBullet {
    pub fn new(ang : Rad<f32>, pos : Point2<f32>) -> Self {
        TestBullet {
            ang, pos,
            lifetime : PLAYER_BULLET_LIFE_LENG,
        }
    }

    pub fn update(&mut self, hive : &mut Hive) {
        use crate::collision::*;

        assert!(self.lifetime > 0);

        let (s, c) = (self.ang + Rad(std::f32::consts::FRAC_PI_2)).sin_cos();
        self.pos += vec2(c, s) * PLAYER_BULLET_STEP_LENGTH;
        self.lifetime -= 1;

        let my_body = collision_models::consts::BulletTester.apply_transform(&self.transform());
        let my_aabb = my_body.aabb();

        for enemy in hive.enemies.iter_mut() {
            if self.lifetime == 0 { break }

            let enemy_body = enemy.phys_body();
            let enemy_aabb = enemy_body.aabb();
        
            if *enemy.hp() > 0 && enemy_aabb.collision_test(my_aabb) && enemy_body.check_collision(&my_body) {
                *enemy.hp_mut() -= 1;
                self.lifetime = 0;
            } 
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
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_angle_z(self.ang) * Matrix4::from_nonuniform_scale(TESTER_BULLET_SIZE.0, TESTER_BULLET_SIZE.1, 1.0f32)
    }
}

pub struct Player {
    pub ang : Rad<f32>,
    pub pos : Point2<f32>,
    pub bullets : MemoryChunk<TestBullet>,
    pub recoil : u64,
    pub speed : u64,
}

impl Player {
    pub fn new() -> Self {
        Player {
            recoil : 0,
            ang : Rad(0.0f32),
            pos : Point2 { x : 0.0f32, y : 0.0f32 },
            bullets : MemoryChunk::with_capacity(ENEMY_BULLET_LIMIT),
            speed : 0,
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
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_angle_z(self.ang) * Matrix4::from_nonuniform_scale(PLAYER_SIZE.0, PLAYER_SIZE.1, 1.0f32)
    }

    pub fn update(&mut self, ang : Rad<f32>) {
        self.recoil = self.recoil.saturating_sub(1);
        self.ang = ang;

        let (s, c) = self.ang.sin_cos();
        self.pos += (self.speed as f32) * 0.01f32 * vec2(-s, c);     
    }

    pub fn update_bullets(&mut self, hive : &mut Hive) {
        self.bullets.iter_mut()
        .for_each(|x| x.update(hive));

        self.bullets.retain(|x| x.lifetime > 0);
    }

    pub fn shoot(&mut self) {
        if self.recoil == 0 {
            self.recoil = PLAYER_BULLET_RECOIL;
            self.bullets.push(TestBullet::new(self.ang, self.pos));
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
