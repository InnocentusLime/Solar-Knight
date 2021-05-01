use std::time::Duration;

use glium::VertexBuffer;

use cgmath_ext::rotate_vector_oy;
use super::core::{ Core, Team };
use sys_api::basic_graphics_data::SpriteData;
use std_ext::{ collections::memory_chunk::MemoryChunk, duration_ext::* };
use sys_api::graphics_init::PLAYER_BULLET_LIMIT;
use cgmath_ext::matrix3_from_translation;

use cgmath::{ Point2, Vector2, Matrix3, Matrix4, EuclideanSpace, InnerSpace, point2, vec2 };

pub const TESTER_BULLET_SIZE : (f32, f32) = (0.06f32, 0.09f32);

// TODO macrofied-bullet construction tool

/// The bullet kind. Can hold data
#[derive(Clone, Copy, Debug)]
pub enum BulletKind {
    TestBullet,
    LaserBall,
//  HomingBullet(target),
}

#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pos : Point2<f32>,
    direction : Vector2<f32>,
    kind : BulletKind,
    lifetime : Duration,
    team : Team,
}

impl Bullet {
    pub fn size(&self) -> (f32, f32) {
        match self.kind {
            BulletKind::TestBullet => (0.06f32, 0.09f32),
            BulletKind::LaserBall => (0.03f32, 0.03f32),
        }
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        let size = self.size();

        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * 
        Matrix4::new(
            self.direction.y, -self.direction.x, 0.0f32, 0.0f32,
            self.direction.x, self.direction.y, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ) * 
        Matrix4::from_nonuniform_scale(size.0, size.1, 1.0f32)
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
    pub fn tester_bullet(
        pos : Point2<f32>, 
        direction : Vector2<f32>,
        team : Team,
    ) -> Bullet {
        Bullet {
            pos,
            team,
            direction,
            kind : BulletKind::TestBullet,
            lifetime : Duration::from_secs(3),
        }
    }
    
    #[inline]
    pub fn laser_ball(
        pos : Point2<f32>, 
        direction : Vector2<f32>,
        team : Team,
    ) -> Bullet {
        Bullet {
            pos,
            team,
            direction,
            kind : BulletKind::LaserBall,
            lifetime : Duration::from_secs(3),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gun {
    offset : Vector2<f32>,
    bullet_maker : fn(Point2<f32>, Vector2<f32>, Team) -> Bullet,
    recoil : Duration,
    timer : Duration,
    direction : Vector2<f32>,
}

impl Gun {
    pub fn new(
        offset : Vector2<f32>,
        bullet_maker : fn(Point2<f32>, Vector2<f32>, Team) -> Bullet,
        recoil : Duration,
        direction : Vector2<f32>,
    ) -> Self {
        // FIXME direction check

        Gun {
            offset,
            bullet_maker,
            recoil,
            timer : <Duration as DurationExt>::my_zero(),
            direction,
        }
    }
            
    #[inline]
    pub fn can_shoot(&self) -> bool {
        self.timer.my_is_zero()
    }
            
    pub fn shoot(&mut self, owner : &crate::core::Core) -> Option<crate::gun::Bullet> {
        if self.can_shoot() {
            self.timer = self.recoil;
            let off = rotate_vector_oy(owner.direction(), self.offset);
            let bullet_dir = rotate_vector_oy(owner.direction(), self.direction);

            Some(
                (self.bullet_maker)(
                    owner.pos + off,
                    bullet_dir,
                    owner.team(),
                )
            )
        } else { None }
    }
            
    pub fn update(&mut self, _core : &mut crate::core::Core, dt : std::time::Duration) {
        self.timer = self.timer.my_saturating_sub(dt);
    }
}

impl Default for Gun {
    fn default() -> Gun {
        Gun::new(
            vec2(0.0f32, 0.0f32),
            Bullet::tester_bullet,
            <Duration as DurationExt>::my_zero(),
            vec2(0.0f32, 1.0f32)
        )
    }
}

pub trait TargetSystem<'a> {
    type Iter : Iterator<Item = &'a mut Core>;

    fn entity_iterator(&'a mut self) -> Self::Iter;
}

pub struct BulletSystem {
    mem : MemoryChunk<Bullet>,
}

impl BulletSystem {
    pub fn new() -> Self {
        BulletSystem {
            mem : MemoryChunk::with_capacity(PLAYER_BULLET_LIMIT),
        }
    }

    pub fn spawn(&mut self, bullet : Bullet) {
        self.mem.push(bullet);
    }

    // FIXME just iterating over all enemies probably sucks.
    // TODO the interface to the hive (`I` must also have a way to random-access enemies)
    pub fn update<I>(&mut self, c : &mut I, dt : Duration) 
    where
        for<'a> I : TargetSystem<'a>
    {
        use collision::*;
        use std_ext::*;
        use crate::collision_models;

        self.mem.iter_mut()
        .for_each(
            |bullet| {
                use crate::constants::VECTOR_NORMALIZATION_RANGE;

                debug_assert!((bullet.direction.magnitude() - 1.0f32) < VECTOR_NORMALIZATION_RANGE);

                bullet.lifetime = bullet.lifetime.my_saturating_sub(dt);

                // Update bullet data and damage on collision
                match bullet.kind {
                    // TestBullet's move towards with speed 
                    // equal to 4.0.
                    BulletKind::TestBullet => {
                        bullet.pos += (4.0f32 * dt.as_secs_f32()) * bullet.direction;
        
                        let my_body = collision_models::consts::BulletTester.apply_transform(&bullet.transform());
                        let my_aabb = my_body.aabb();

                        for target in c.entity_iterator() {                
                            if bullet.lifetime.my_is_zero() { break }

                            let target_body = target.phys_body();
                            let target_aabb = target_body.aabb();
            
                            if 
                                target.team() != bullet.team &&
                                target.hp() > 0 && 
                                target_aabb.collision_test(my_aabb) && 
                                target_body.check_collision(&my_body)
                            {
                                target.damage(1);
                                bullet.lifetime = <Duration as DurationExt>::my_zero();
                            } 
                        }
                    },        
                    BulletKind::LaserBall => {
                        bullet.pos += (0.7f32 * dt.as_secs_f32()) * bullet.direction;
        
                        let my_body = collision_models::consts::LaserBall.apply_transform(&bullet.transform());
                        let my_aabb = my_body.aabb();

                        for target in c.entity_iterator() {                
                            if bullet.lifetime.my_is_zero() { break }

                            let target_body = target.phys_body();
                            let target_aabb = target_body.aabb();
            
                            if 
                                target.team() != bullet.team &&
                                target.hp() > 0 && 
                                target_aabb.collision_test(my_aabb) && 
                                target_body.check_collision(&my_body)
                            {
                                target.damage(1);
                                bullet.lifetime = <Duration as DurationExt>::my_zero();
                            } 
                        }
                    },        
                }
            } 
        );

        self.mem.retain(
            |bullet| {
                // Determine what is illegal
                // for the bullet to live
                match bullet.kind {
                    BulletKind::TestBullet => !bullet.lifetime.my_is_zero(),
                    BulletKind::LaserBall => !bullet.lifetime.my_is_zero(),
                }
            }
        );

        // Post processing
        self.mem.iter_mut()
        .for_each(
            |_bullet| {
                // Nothing for now
            }
        );
    }

    pub fn fill_buffer(&self, buff : &mut VertexBuffer<SpriteData>) {
        let mut ptr = buff.map_write();

        if ptr.len() < PLAYER_BULLET_LIMIT { panic!("Buffer too small"); }

        for i in 0..ptr.len() { 
            use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
            
            ptr.set(i, ZEROED_SPRITE_DATA);
        }

        self.mem.iter()
        .enumerate()
        .for_each(|(i, x)| {
            let m = x.model_mat();
            //dbg!(i); dbg!(m);
            
            let dat =
                SpriteData {
                    mat_col1 : m.x.into(),
                    mat_col2 : m.y.into(),
                    mat_col3 : m.z.into(),
                    mat_col4 : m.w.into(),
                    texture_bottom_left : [0.0f32, 0.0f32],
                    width_height : [1.0f32, 1.0f32],
                    color : [1.0f32, 1.0f32, 1.0f32, 1.0f32],
                }
            ;
            
            ptr.set(i, dat);
        });
    }
}
