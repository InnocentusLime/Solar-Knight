use cgmath::{ vec2, point2 };
use tinyvec::array_vec;

pub use systems::{
    ship_transform::Transform,
    ship_gun::{ Gun, Guns, BulletKind },
    ship_engine::{ Engine, Engines },
    physics::PhysicsData,
    hp_system::HpInfo,
    square_map::SquareMapNode,
    collision_check::CollisionInfo,
    teams::Team,
};
use systems::{ declare_object, declare_observers };
use crate::ai_machine::AiTag;
use crate::render::RenderInfo;
use crate::storage::Storage;

use std::time::Duration;

declare_object! {
    #[derive(Clone, Copy)]
    pub object Ship {
        team : Team,
        transform : Transform,
        guns : Guns,
        engines : Engines,
        physics : PhysicsData,
        hp_info : HpInfo,
        collision : CollisionInfo,
        square_map_node : SquareMapNode,
        render_info : RenderInfo,
        think : AiTag,
    }
}

pub type ShipStorage = Storage<Ship>;
declare_observers! {
    type Host = ShipStorage;

    pub observer SpawningObserverPack {
        square_map : systems::square_map::SquareMap,
    }
    
    pub observer DeletionObserverPack {
        square_map : systems::square_map::SquareMap,
        attach_sys : systems::ship_attachment::AttachmentSystem,
        bullet_sys : systems::ship_gun::BulletSystem,
    }
    
    pub observer MutationObserverPack {
        square_map : systems::square_map::SquareMap,
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
            prefab : Ship {
                team : Team::Earth,
                transform : Transform::new(point2(0.0f32, 0.0f32)),
                guns : Guns { guns : array_vec![_ => 
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::TestBullet, Duration::from_millis(300), vec2(0.0f32, 1.0f32)),
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::LaserBeam, Duration::from_secs(4), vec2(0.0f32, 1.0f32)),
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::SpinningLaser, Duration::from_secs(2), vec2(0.0f32, 1.0f32)),
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::HomingMissle, Duration::from_millis(600), vec2(0.0f32, 1.0f32)),
                ]},
                engines : Engines { engines : array_vec![_ => 
                    Engine::new(vec2(0.0f32, 1.0f32), 1, 5.0f32, 0),
                    Engine::new(vec2(0.0f32, 1.0f32), 1, 1200.0f32, 0),
                ]},
                physics : PhysicsData::new(5.0f32),
                hp_info : HpInfo::new(3),
                collision : CollisionInfo {
                    model : systems::collision_check::CollisionModelIndex::Player,
                },
                square_map_node : SquareMapNode::new(),
                render_info : RenderInfo { enemy_base_texture : false },
                think : AiTag::None,
            },
        }
    }

    pub fn turret_ship() -> Self {
        TemplateTableEntry {
            name : "Turret enemy".to_owned(),
            prefab : Ship {
                team : Team::Hive,
                transform : Transform::new(point2(0.0f32, 0.0f32)),
                guns : Guns { guns : array_vec![_ => 
                    Gun::new(vec2(0.0f32, 0.0f32), BulletKind::LaserBall, Duration::from_millis(400), vec2(0.0f32, 1.0f32))
                ]},
                engines : Engines { engines : array_vec![] },
                physics : PhysicsData::new(100.0f32),
                hp_info : HpInfo::new(3),
                collision : CollisionInfo {
                    model : systems::collision_check::CollisionModelIndex::Player,
                },
                square_map_node : SquareMapNode::new(),
                render_info : RenderInfo { enemy_base_texture : false },
                think : AiTag::Turret,
            },
        }
    }

    // TODO heavy body should be bigger
    // and have different graphics
    // TODO heavy should target earth in advance
    pub fn heavy_body() -> Self {
        TemplateTableEntry {
            name : "Heavy's body".to_owned(),
            prefab : Ship {
                team : Team::Hive,
                transform : Transform::new(point2(0.0f32, 0.0f32)),
                guns : Guns { guns : array_vec![] },
                engines : Engines { engines : array_vec![_ => 
                    Engine::new(vec2(0.0f32, 1.0f32), 1, 1.0f32, 1),
                ]},
                physics : PhysicsData::new(100.0f32),
                hp_info : HpInfo::new(10),
                collision : CollisionInfo {
                    model : systems::collision_check::CollisionModelIndex::Player,
                },
                square_map_node : SquareMapNode::new(),
                render_info : RenderInfo { enemy_base_texture : true },
                think : AiTag::HeavyBody,
            },
        }
    }
    
    pub fn fly_ship() -> Self {
        TemplateTableEntry {
            name : "Fly".to_owned(),
            prefab : Ship {
                team : Team::Hive,
                transform : Transform::new(point2(0.0f32, 0.0f32)),
                guns : Guns { guns : array_vec![] },
                engines : Engines { engines : array_vec![_ => 
                    Engine::new(vec2(0.0f32, 1.0f32), 2, 1.2f32, 1),
                ]},
                physics : PhysicsData::new(1.5f32),
                hp_info : HpInfo::new(1),
                collision : CollisionInfo {
                    model : systems::collision_check::CollisionModelIndex::Player,
                },
                square_map_node : SquareMapNode::new(),
                render_info : RenderInfo { enemy_base_texture : false },
                think : AiTag::Fly,
            },
        }
    }
}

pub struct TemplateTable(Vec<TemplateTableEntry>);

impl TemplateTable {
    #[inline]
    pub fn new() -> Self {
        TemplateTable(vec![
            TemplateTableEntry::player_ship(),
            TemplateTableEntry::turret_ship(),
            TemplateTableEntry::heavy_body(),
            TemplateTableEntry::fly_ship(),
        ])
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn get_ship(&self, idx : usize) -> Option<&Ship> {
        self.0.get(idx).map(|x| &x.prefab)
    }

    #[inline]
    pub fn get_name(&self, idx : usize) -> Option<&str> {
        self.0.get(idx).map(|x| x.name.as_str())
    }

    #[inline]
    pub fn spawn_template<Observer>(
        &self, 
        idx : usize, 
        obs : &mut systems::systems_core::Observation<Observer, ShipStorage>
    ) -> usize
    where
        Observer : systems::systems_core::SpawningObserver<ShipStorage>,
    {
        obs.spawn(*self.get_ship(idx).unwrap())
    }
}
