use glium::VertexBuffer;
use cgmath::{ EuclideanSpace, Matrix4, Angle, Rad, Point2, vec2 };

use crate::graphics_init::ENEMY_BULLET_LIMIT;
use crate::basic_graphics_data::SpriteData;
use crate::containers::MemoryChunk;

/// Ship's speed is `1/(10 - x)`. Where `x` is a value which increases when `space` is down
/// Right now `x` is capped at `8.0f32`

const PLAYER_BULLET_STEP_LENGTH : f32 = 0.05f32;
const PLAYER_BULLET_LIFE_LENG : u64 = 300;
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

    pub fn update(&mut self) {
        if self.lifetime == 0 { panic!("You tried to update a dead bullet, dude") }

        let (s, c) = (self.ang + Rad(std::f32::consts::FRAC_PI_2)).sin_cos();
        self.pos += vec2(c, s) * PLAYER_BULLET_STEP_LENGTH;
        self.lifetime -= 1;
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_angle_z(self.ang)
    }
}

pub struct Player {
    pub ang : Rad<f32>,
    pub pos : Point2<f32>,
    pub bullets : MemoryChunk<TestBullet>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            ang : Rad(0.0f32),
            pos : Point2 { x : 0.0f32, y : 0.0f32 },
            bullets : MemoryChunk::with_capacity(ENEMY_BULLET_LIMIT),
        }
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * Matrix4::from_angle_z(self.ang)
    }

    pub fn update(&mut self, movement : bool, ang : Rad<f32>) {
        self.ang = ang;

        if movement {
            let (s, c) = self.ang.sin_cos();
            self.pos += 0.01f32 * vec2(-s, c); 
        }
        
        self.bullets.iter_mut()
        .for_each(|x| x.update());

        self.bullets.retain(|x| x.lifetime > 0);
    }

    pub fn shoot(&mut self) {
        self.bullets.push(TestBullet::new(self.ang, self.pos));
    }

    pub fn fill_bullet_buffer(&self, buff : &mut VertexBuffer<SpriteData>) {
        let mut bullet_ptr = buff.map_write();

        if bullet_ptr.len() < ENEMY_BULLET_LIMIT { panic!("Buffer too small"); }

        for i in 0..bullet_ptr.len() { 
            use crate::basic_graphics_data::ZEROED_SPRITE_DATA;
            
            bullet_ptr.set(i, ZEROED_SPRITE_DATA);
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
                }
            ;
            
            bullet_ptr.set(i, dat);
        });
    }
}
