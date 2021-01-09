pub mod brute;
pub mod tester;

use std::time::Duration;

use glium::VertexBuffer;
use cgmath::{ Matrix3, Matrix4 };

use crate::graphics_init::ENEMY_LIMIT;
use crate::collision_models::CollisionModel;
use crate::basic_graphics_data::SpriteData;
use crate::states::main_game::player::Player;
use crate::containers::MemoryChunk;

const ENEMY_ALLOC : usize = ENEMY_LIMIT;

pub enum Enemy {
    Brute(brute::Brute),
    Tester(tester::Tester),
}

impl Enemy {
    pub fn update(&mut self, player : &Player, dt : Duration) {
        match self {
            Enemy::Brute(_) => (),
            Enemy::Tester(x) => x.update(player, dt),
        }
    }

    pub fn hp_mut(&mut self) -> &mut u64 {
        match self {
            Enemy::Brute(_) => unimplemented!(),
            Enemy::Tester(x) => x.hp_mut(), 
        }
    }
    
    pub fn hp(&self) -> &u64 {
        match self {
            Enemy::Brute(_) => unimplemented!(),
            Enemy::Tester(x) => x.hp(), 
        }
    }

    pub fn phys_body(&self) -> CollisionModel  {
        match self {
            Enemy::Brute(_) => unimplemented!(),
            Enemy::Tester(x) => x.collision_model(),
        }
    }

    pub fn model_mat(&self) -> Matrix4<f32> {
        match self {
            Enemy::Brute(_) => unimplemented!(),
            Enemy::Tester(x) => x.model_mat(),
        }
    }
}

pub struct Hive {
    enemies : MemoryChunk<Enemy>, 
}

impl Hive {
    pub fn new() -> Self {
        Hive {
            enemies : MemoryChunk::with_capacity(ENEMY_ALLOC),
        }
    }

    #[inline]
    pub fn alive_count(&self) -> usize { self.enemies.len() }

    pub fn update(&mut self, player : &Player, dt : Duration) {
        self.enemies
        .iter_mut()
        .for_each(|x| x.update(player, dt));

        self.enemies.retain(|x| *x.hp() > 0);
    }

    pub fn fill_enemy_buffer(&self, buff : &mut VertexBuffer<SpriteData>) {
        let mut ptr = buff.map_write();

        if ptr.len() < ENEMY_LIMIT { panic!("Buffer too small"); }

        for i in 0..ptr.len() { 
            use crate::basic_graphics_data::ZEROED_SPRITE_DATA;
            
            ptr.set(i, ZEROED_SPRITE_DATA);
        }

        self.enemies.iter()
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

    pub fn spawn(&mut self, enemy : Enemy) {
        self.enemies.push(enemy);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Enemy> { self.enemies.iter() }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Enemy> { self.enemies.iter_mut() }
}
