use std::any::{ Any, TypeId };
use std::time::Duration;

use crate::earth::Earth;
use crate::core::{ Core, Team };
use crate::engine::Engine;
use crate::gun::{ BulletSystem, TargetSystem, Gun };

use std_ext::ExtractResultMut;
use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;

use glium::VertexBuffer;
use tinyvec::ArrayVec;
use cgmath::{ Matrix4, EuclideanSpace, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

pub static mut FRICTION_KOEFF : f32 = 0.5f32;

#[derive(Clone, Copy)]
pub struct Ship {
    pub render : fn(&Self, &mut SpriteDataWriter),
    pub think : fn(
        me : &mut Self,
        others : &ExtractResultMut<Self>,
        bullet_system : &mut BulletSystem,
        earth : &Earth,
        dt : Duration
    ),
    pub core : Core,
    pub engines : ArrayVec<[Engine; 5]>,
    pub guns : ArrayVec<[Gun; 5]>,
}

impl Ship {
    #[inline]
    pub fn model_mat(&self, size : (f32, f32)) -> Matrix4<f32> {
        let direction = self.core.direction();

        Matrix4::from_translation(self.core.pos.to_vec().extend(0.0f32)) * 
        Matrix4::new(
            direction.y, -direction.x, 0.0f32, 0.0f32,
            direction.x, direction.y, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ) * 
        Matrix4::from_nonuniform_scale(size.0, size.1, 1.0f32)
    }
        
    pub fn new(
        core : Core, 
        think : fn(
            me : &mut Self,
            others : &ExtractResultMut<Self>,
            bullet_system : &mut BulletSystem,
            earth : &Earth,
            dt : Duration
        ),
        render : fn(&Self, &mut SpriteDataWriter),
        engines : ArrayVec<[Engine; 5]>,
        guns : ArrayVec<[Gun; 5]>,
    ) -> Self {
        Ship { 
            render,
            think,
            core,
            engines,
            guns,
        }
    }
}

pub struct Battlefield {
    pub earth : Earth,
    mem : Vec<Ship>,
}

impl Battlefield {
    pub fn new() -> Battlefield {
        Battlefield {
            mem : Vec::new(),
            earth : Earth::new(),
        }
    }

    pub fn update(&mut self, dt : Duration) {
        use crate::constants::VECTOR_NORMALIZATION_RANGE;

        let friction_koeff = unsafe { FRICTION_KOEFF };

        self.earth.update(dt);

        self.mem.iter_mut()
        .for_each(
            |c| {
                c.core.force = vec2(0.0f32, 0.0f32);

                let (core, engines, guns) = (&mut c.core, &mut c.engines, &mut c.guns);
                engines.iter_mut().for_each(|x| x.update(core, dt));
                guns.iter_mut().for_each(|x| x.update(core, dt));

                if 
                    abs_diff_ne!(c.core.velocity.magnitude(), 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) 
                {
                    c.core.force -= 0.24f32 * c.core.velocity.magnitude() * c.core.velocity;
                }
                c.core.velocity += (dt.as_secs_f32() / c.core.mass) * c.core.force;
                c.core.pos += dt.as_secs_f32() * c.core.velocity;
            }
        );

        self.mem.retain(|x| x.core.is_alive() || x.core.team() == Team::Earth);
    }
            
    pub fn think(&mut self, bullet_system : &mut BulletSystem, dt : Duration) {
        use std_ext::*;

        for i in 0..self.mem.len() {
            let (extract, elem) = self.mem.as_mut_slice().extract_mut(i);

            if elem.core.is_alive() {
                (elem.think)(elem, &extract, bullet_system, &self.earth, dt);
            }
        }
    }
    
    pub fn spawn(&mut self, ship : Ship) {
        self.mem.push(ship);
    }
            
    pub fn fill_buffer(&self, buff : &mut VertexBuffer<SpriteData>) {
        use sys_api::graphics_init::{ ENEMY_LIMIT };
                
        let mut ptr = buff.map_write();

        if ptr.len() < ENEMY_LIMIT { panic!("Buffer too small"); }

        for i in 0..ptr.len() { 
            use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
            
            ptr.set(i, ZEROED_SPRITE_DATA);
        }

        let mut writer = SpriteDataWriter::new(ptr);
        self.mem.iter().for_each(|x| (x.render)(x, &mut writer));
    }

    #[inline]
    pub fn get(&self, id : usize) -> Option<&Ship> { self.mem.get(id) }
    
    #[inline]
    pub fn get_mut(&mut self, id : usize) -> Option<&mut Ship> { self.mem.get_mut(id) }
}
        
use std::slice::IterMut;
use std::iter::Map;
impl<'a> TargetSystem<'a> for Battlefield {
    type Iter = Map<IterMut<'a, Ship>, fn(&mut Ship) -> &mut Core>;

    fn entity_iterator(&'a mut self) -> Self::Iter {
        self.mem.iter_mut().map(|x| &mut x.core)
    }
}
