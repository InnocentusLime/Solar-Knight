use std::any::{ Any, TypeId };
use std::time::Duration;

use crate::earth::Earth;
use crate::core::{ Core, Team };
use crate::engine::Engine;
use crate::gun::{ BulletSystem, Gun, BulletKind };

use std_ext::ExtractResultMut;
use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;
use crate::constants::VECTOR_NORMALIZATION_RANGE;
use cgmath_ext::rotate_vector_ox;
use crate::render::RenderInfo;
use crate::collision_models::model_indices::*;
use crate::ai_machine::*;

use slab::Slab;
use glium::VertexBuffer;
use tinyvec::ArrayVec;
use tinyvec::array_vec;
use serde::{ Serialize, Deserialize };
use cgmath::{ Point2, Matrix4, EuclideanSpace, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

pub static mut FRICTION_KOEFF : f32 = 0.5f32;

// TODO probably should place the gun limit into
// a separate constant
#[derive(Clone, Copy)]
pub struct Ship {
    pub render : RenderInfo,
    // TODO make it part of the new ai system
    pub think : Option<RoutineId>,
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
        think : Option<RoutineId>,
        render : RenderInfo,
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

pub struct TemplateTableEntry {
    pub name : String,
    pub prefab : Ship,
}

// Temporary code. In the future we want
// to serialize all that jazz and have it
// in the files.
// Data driven design 101 ;)
impl TemplateTableEntry {
    pub fn player_ship() -> Self {
        TemplateTableEntry {
            name : "Player ship".to_owned(),
            prefab : Ship::new(
                Core::new(3, 5.0f32, CollisionModelIndex::Player, Team::Earth),
                None,
                RenderInfo { enemy_base_texture : false },
                array_vec![_ => Engine::new(vec2(0.0f32, 1.0f32), 1, 5.0f32, 0)],
                array_vec![_ => Gun::new(vec2(0.0f32, 0.0f32), BulletKind::TestBullet, Duration::from_millis(300), vec2(0.0f32, 1.0f32))],
            ),
        }
    }

    pub fn turret_ship() -> Self {
        TemplateTableEntry {
            name : "Turret enemy".to_owned(),
            prefab : Ship::new(
                Core::new(3, 100.0f32, CollisionModelIndex::Player, Team::Hive),
                Some(RoutineId(0)),
                RenderInfo { enemy_base_texture : false },
                array_vec![],
                array_vec![_ => Gun::new(vec2(0.0f32, 0.0f32), BulletKind::LaserBall, Duration::from_millis(400), vec2(0.0f32, 1.0f32))],
            ),
        }
    }

    // TODO heavy body should be bigger
    // and have different graphics
    // TODO heavy should target earth in advance
    pub fn heavy_body() -> Self {
        TemplateTableEntry {
            name : "Heavy's body".to_owned(),
            prefab : Ship::new(
                Core::new(10, 100.0f32, CollisionModelIndex::Player, Team::Hive),
                Some(RoutineId(1)),
                RenderInfo { enemy_base_texture : true },
                array_vec![_ => Engine::new(vec2(0.0f32, 1.0f32), 1, 1.0f32, 0)],
                array_vec![],
            ),
        }
    }
    
    pub fn fly_ship() -> Self {
        TemplateTableEntry {
            name : "Fly".to_owned(),
            prefab : Ship::new(
                Core::new(1, 1.5f32, CollisionModelIndex::Player, Team::Hive),
                Some(RoutineId(2)),
                RenderInfo { enemy_base_texture : false },
                array_vec![_ => Engine::new(vec2(0.0f32, 1.0f32), 2, 1.2f32, 0)],
                array_vec![],
            ),
        }
    }
}

pub struct Battlefield {
    uid_counter : u128,
    pub earth : Earth,
    mem : Slab<Ship>,
    pub ai_machine : AiMachine,
    pub template_table : Vec<TemplateTableEntry>,
}

impl Battlefield {
    pub fn new() -> Battlefield {
        Battlefield {
            uid_counter : 0,
            mem : Slab::new(),
            earth : Earth::new(),
            ai_machine : AiMachine::new(),
            template_table : vec![
                TemplateTableEntry::player_ship(),
                TemplateTableEntry::turret_ship(),
                TemplateTableEntry::heavy_body(),
                TemplateTableEntry::fly_ship(),
            ],
        }
    }

    pub fn update(&mut self, dt : Duration) {
        use crate::constants::VECTOR_NORMALIZATION_RANGE;

        let friction_koeff = unsafe { FRICTION_KOEFF };

        self.earth.update(dt);

        self.mem.iter_mut()
        .for_each(
            |(_, c)| {
                c.core.force = vec2(0.0f32, 0.0f32);

                let (core, engines, guns) = (&mut c.core, &mut c.engines, &mut c.guns);
                engines.iter_mut().for_each(|x| x.update(core, dt));
                guns.iter_mut().for_each(|x| x.update(core, dt));

                if 
                    abs_diff_ne!(c.core.velocity.magnitude(), 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) 
                {
                    c.core.force -= 0.24f32 * c.core.velocity.magnitude() * c.core.velocity;
                }
                let acceleration = c.core.force / c.core.mass;
                c.core.pos += dt.as_secs_f32() * c.core.velocity + dt.as_secs_f32().powi(2) * acceleration / 2.0f32;
                c.core.velocity += dt.as_secs_f32() * acceleration;
            }
        );

        self.mem.retain(|_, x| x.core.is_alive() || x.core.team() == Team::Earth);
    }
   
    pub fn spawn(&mut self, mut ship : Ship) {
        ship.core.set_uid(self.uid_counter);
        self.uid_counter += 1;
        self.mem.insert(ship);
    }

    pub fn spawn_template(&mut self, id : usize) {
        self.spawn(self.template_table[id].prefab);
    }
            
    pub fn fill_buffer(&self, buff : &mut VertexBuffer<SpriteData>) {
        use sys_api::graphics_init::{ ENEMY_LIMIT };
                
        //self.mem.iter().for_each(|(_, x)| (x.render)(x, &mut writer));
    }

    #[inline]
    pub fn get(&self, id : usize) -> Option<&Ship> { self.mem.get(id) }
    
    #[inline]
    pub fn get_mut(&mut self, id : usize) -> Option<&mut Ship> { self.mem.get_mut(id) }
    
    #[inline]
    pub fn len(&self) -> usize { self.mem.len() }
    
    #[inline]
    pub fn capacity(&self) -> usize { self.mem.capacity() }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Ship> {
        self.mem.iter().map(|(_, x)| x)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Ship> {
        self.mem.iter_mut().map(|(_, x)| x)
    }
}
