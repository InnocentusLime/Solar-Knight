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
use crate::square_map::SquareMapNode;
use crate::ai_machine::RoutineId;

use slab::Slab;
use glium::VertexBuffer;
use tinyvec::ArrayVec;
use tinyvec::array_vec;
use serde::{ Serialize, Deserialize };
use cgmath::{ Point2, Matrix4, EuclideanSpace, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

// TODO probably should place the gun limit into
// a separate constant
#[derive(Clone, Copy, Debug)]
pub struct Ship {
    pub render : RenderInfo,
    // TODO make it part of the new ai system
    pub think : Option<RoutineId>,
    pub core : Core,
    pub engines : ArrayVec<[Engine; 5]>,
    pub guns : ArrayVec<[Gun; 5]>,
    pub square_map_node : SquareMapNode,
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
            square_map_node : SquareMapNode::new(),
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
                array_vec![_ => 
                    Engine::new(vec2(0.0f32, 1.0f32), 1, 5.0f32, 0),
                    Engine::new(vec2(0.0f32, 1.0f32), 1, 1200.0f32, 0),
                ],
                array_vec![_ => 
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::TestBullet, Duration::from_millis(300), vec2(0.0f32, 1.0f32)),
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::LaserBeam, Duration::from_secs(4), vec2(0.0f32, 1.0f32)),
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::SpinningLaser, Duration::from_secs(2), vec2(0.0f32, 1.0f32)),
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::HomingMissle, Duration::from_millis(600), vec2(0.0f32, 1.0f32)),
                ],
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
                array_vec![_ => Engine::new(vec2(0.0f32, 1.0f32), 1, 1.0f32, 1)],
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

pub struct Storage {
    mem : Slab<Ship>,
    pub template_table : Vec<TemplateTableEntry>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            mem : Slab::new(),
            template_table : vec![
                TemplateTableEntry::player_ship(),
                TemplateTableEntry::turret_ship(),
                TemplateTableEntry::heavy_body(),
                TemplateTableEntry::fly_ship(),
            ],
        }
    }

    pub(crate) fn spawn(&mut self, ship : Ship) -> usize {
        self.mem.insert(ship)
    }
    
    #[inline]
    pub(crate) fn spawn_template(&mut self, id : usize) -> usize {
        self.spawn(self.template_table[id].prefab)
    }
   
    // TODO migrate to this function
    #[inline]
    pub(crate) fn spawn_template_at(&mut self, id : usize, pos : Point2<f32>) -> usize {
        let mut ship = self.template_table[id].prefab;
        ship.core.pos = pos;
        self.spawn(ship)
    }

    #[inline]
    pub fn get(&self, id : usize) -> Option<&Ship> { self.mem.get(id) }
    
    #[inline]
    fn get_mut(&mut self, id : usize) -> Option<&mut Ship> { self.mem.get_mut(id) }

    #[inline]
    pub(crate) fn delete(&mut self, id : usize) -> Option<Ship> {
        if self.mem.contains(id) { 
            Some(self.mem.remove(id)) 
        } else { None }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Ship> {
        self.mem.iter().map(|(_, x)| x)
    }

    #[inline]
    pub(crate) fn mutate(&mut self) -> MutableStorage { MutableStorage(self) }

    #[inline]
    pub fn capacity(&self) -> usize { self.mem.capacity() }
}

pub struct MutableStorage<'a>(&'a mut Storage);

impl<'a> MutableStorage<'a> {
    #[inline]
    pub fn get(&self, id : usize) -> Option<&Ship> { self.0.get(id) }
    
    #[inline]
    pub fn get_mut(&mut self, id : usize) -> Option<&mut Ship> { self.0.get_mut(id) }
    
    #[inline]
    pub fn capacity(&self) -> usize { self.0.capacity() }
}
