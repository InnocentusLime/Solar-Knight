use std::any::{ Any, TypeId };
use std::time::Duration;

use crate::earth::Earth;
use crate::core::{ Core, Team };
use crate::engine::Engine;
use crate::gun::{ BulletSystem, Gun, Bullet };

use std_ext::ExtractResultMut;
use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;
use crate::constants::VECTOR_NORMALIZATION_RANGE;
use cgmath_ext::rotate_vector_ox;
use crate::render::RenderInfo;
use crate::collision_models::model_indices::*;

use slab::Slab;
use glium::VertexBuffer;
use tinyvec::ArrayVec;
use tinyvec::array_vec;
use serde::{ Serialize, Deserialize };
use cgmath::{ Point2, Matrix4, EuclideanSpace, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

pub static mut FRICTION_KOEFF : f32 = 0.5f32;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RoutineId(pub usize);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CommandId(pub usize);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Target {
    Earth,
    Ship(usize),
// Forgotten for now
//  Current,
}

// TODO make generic
impl Target {
    // TODO all those ship_id checks should
    // go here.
    fn get_pos(
        self,
        storage : &Slab<Ship>, 
        earth : &Earth,
    ) -> Point2<f32> {
        match self {
            Target::Earth => earth.pos(),
            Target::Ship(ship_id) => storage[ship_id].core.pos,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ExecutionControl {
    GoTo(CommandId),
    Done(RoutineId),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CommandError {
    TargetOutOfRange,
    TriedToTargetSelf,
    GunNotPresent,
    EngineNotPresent,
}

// Ai routine is a block of commands which decide ships new state and
// then says which ai routine should be executed next frame
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AiCommand {
    End(RoutineId),
    Noop {
        next : CommandId,
    },
// Forgotten for now
//    PickTarget(Target), 
    IsTargetClose {
        target : Target,
        distance : f32,
        on_success : CommandId,
        on_failure : CommandId,
    },
    CanSeeTarget {
        target : Target,
        view_angle : f32,
        on_success : CommandId,
        on_failure : CommandId,
    },
    RotateTowards {
        target : Target,
        angular_speed : f32,
        next : CommandId,
    },
    // TODO make it branch?
    Shoot {
        gun : usize,
        next : CommandId,
    },
    IncreaseSpeed {
        engine : usize,
        next : CommandId,
    },
    DecreaseSpeed {
        engine : usize,
        next : CommandId,
    },
}

// TODO make generic
impl AiCommand {
    pub fn run(
        self, 
        me : usize,
        storage : &mut Slab<Ship>, 
        bullet_system : &mut crate::gun::BulletSystem,
        earth : &Earth,
        dt : Duration,
    ) -> Result<ExecutionControl, CommandError> {
        // TODO become aware of self-reference
        // TODO become aware of out-of-range target
        match self {
            AiCommand::End(new_routine) => Ok(ExecutionControl::Done(new_routine)),
            AiCommand::Noop {
                next,
            } => Ok(ExecutionControl::GoTo(next)),
            AiCommand::IsTargetClose {
                target,
                distance,
                on_success,
                on_failure,
            } => {
                let me = storage.get(me).unwrap();
                let target = target.get_pos(storage, earth);
                let dir_vec = (target - me.core.pos).normalize();
                if dir_vec.magnitude() <= distance {
                    Ok(ExecutionControl::GoTo(on_success))
                } else {
                    Ok(ExecutionControl::GoTo(on_failure))
                }
            },
            AiCommand::CanSeeTarget {
                target,
                view_angle,
                on_success,
                on_failure,
            } => {
                let me = storage.get(me).unwrap();
                let target = target.get_pos(storage, earth);
                let dir_vec = target - me.core.pos;
                let ang = me.core.direction().angle(dir_vec);
                if ang.0.abs() <= view_angle / 2.0f32 {
                    Ok(ExecutionControl::GoTo(on_success))
                } else {
                    Ok(ExecutionControl::GoTo(on_failure))
                }
            },
            AiCommand::RotateTowards {
                target,
                angular_speed,
                next,
            } => {
                let target = target.get_pos(storage, earth);
                let me = storage.get_mut(me).unwrap();
                let dir_vec = (target - me.core.pos).normalize();
                let ang = me.core.direction().angle(dir_vec);

                if abs_diff_ne!(ang.0, 0.0f32, epsilon = VECTOR_NORMALIZATION_RANGE) {
                    let (c, s) =
                        if ang.0 > 0.0f32 {
                            ((angular_speed * dt.as_secs_f32()).cos(), (angular_speed * dt.as_secs_f32()).sin())
                        } else {
                            ((angular_speed * dt.as_secs_f32()).cos(), -(angular_speed * dt.as_secs_f32()).sin())
                        } 
                    ;
                    me.core.set_direction(rotate_vector_ox(me.core.direction(), vec2(c, s)));
                }

                Ok(ExecutionControl::GoTo(next))
            },
            AiCommand::Shoot {
                gun,
                next,
            } => {
                let me_id = me;
                let me = storage.get_mut(me).unwrap();
                match me.guns.get_mut(gun) {
                    Some(gun) => {
                        gun.shoot(&me.core) 
                        .map_or(
                            (),
                            |x| bullet_system.spawn(x, me_id)
                        );
                        Ok(ExecutionControl::GoTo(next))
                    },
                    None => Err(CommandError::GunNotPresent),
                }
            },
            AiCommand::IncreaseSpeed {
                engine,
                next,
            } => {
                let me = storage.get_mut(me).unwrap();
                match me.engines.get_mut(engine) {
                    Some(engine) => {
                        engine.increase_speed();
                        Ok(ExecutionControl::GoTo(next))
                    },
                    None => Err(CommandError::EngineNotPresent),
                }
            },
            AiCommand::DecreaseSpeed {
                engine,
                next,
            } => {
                let me = storage.get_mut(me).unwrap();
                match me.engines.get_mut(engine) {
                    Some(engine) => {
                        engine.decrease_speed();
                        Ok(ExecutionControl::GoTo(next))
                    },
                    None => Err(CommandError::EngineNotPresent),
                }
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RoutineError {
    TimeLimit,
    CommandError {
        command : CommandId,
        error : CommandError,
    },
    StartOutOfRange,
    CommandIdOutOfRange,
}

// NOTE
// Ai routines shouldn't be complex
// Some complex decisions should be 
// solved by a system which exists above
// and operates through reacting to
// events which get sent to it by the game's logic
// those decision systems should then
// figure out which ai routine should
// the ship (NPC) start to run
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiRoutine {
    commands : Vec<AiCommand>,
}

impl AiRoutine {
    // The limit of how much `GoTo`'s can be executed.
    // Which is basically how many commands are allowed
    // to be executed.
    const COMMAND_LIMIT : Option<usize> = Some(10);

    pub fn run(
        &self,
        me : usize,
        storage : &mut Slab<Ship>, 
        bullet_system : &mut crate::gun::BulletSystem,
        earth : &Earth,
        dt : Duration,
    ) -> Result<RoutineId, RoutineError> {
        // Still including this... For the invariant and the release builds.
        // The above assert will be later removed and some code will be added
        // to make the engine react meaningfully to the error
        if self.commands.len() == 0 { return Err(RoutineError::StartOutOfRange); }

        let mut command_limit = Self::COMMAND_LIMIT;
        let mut routine_status = ExecutionControl::GoTo(CommandId(0));

        // Invariant of the loop: 
        // 1. `command_limit` is either `None` or `Some(x)`, where `x > 0`
        // 2. `routine_status` is `Ok(ExecutionControl::GoTo(cmd))` and `cmd` is a valid command ID
        while let ExecutionControl::GoTo(current) = routine_status {
            // Check if the command limit has been exceeded
            if let Some(counter) = command_limit.as_mut() {
                *counter = counter.checked_sub(1).ok_or(RoutineError::TimeLimit)?;
            }
            
            match self.commands.get(current.0) {
                Some(cmd) =>
                    routine_status =
                    cmd.run(
                        me,
                        storage, 
                        bullet_system,
                        earth,
                        dt
                    ).map_err(
                        |error| RoutineError::CommandError { error, command : current }
                    )?
                ,
                None => return Err(RoutineError::CommandIdOutOfRange),
            }
        }

        match routine_status {
            ExecutionControl::Done(routine_id) => Ok(routine_id),
            _ => unreachable!("Illegal routine status set after the execution loop"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiMachine {
    routines : Vec<AiRoutine>,
}

impl AiMachine {
    pub fn new() -> Self {
        AiMachine {
            routines : vec![
                AiRoutine {
                    commands : vec![
                        AiCommand::IsTargetClose {
                            target : Target::Ship(0),
                            distance : 1.5f32,
                            on_success : CommandId(1),
                            on_failure : CommandId(4),
                        },
                        AiCommand::RotateTowards {
                            target : Target::Ship(0),
                            angular_speed : std::f32::consts::TAU / 4.032,
                            next : CommandId(2),
                        },
                        AiCommand::CanSeeTarget {
                            target : Target::Ship(0),
                            view_angle : std::f32::consts::TAU / 8.0f32,
                            on_success : CommandId(3),
                            on_failure : CommandId(4),
                        },
                        AiCommand::Shoot {
                            gun : 0,
                            next : CommandId(4),
                        },
                        AiCommand::End(RoutineId(0)),
                    ], 
                }
            ],
        }
    }

    // TODO return a Result
    // FIXME get rid off the checks by making the code
    // indepent from the ship itself
    pub fn think_for(
        &self,
        me : usize,
        storage : &mut Slab<Ship>, 
        bullet_system : &mut crate::gun::BulletSystem,
        earth : &Earth,
        dt : Duration,
    ) {
        let ship = storage.get_mut(me).unwrap();
        let new_think = {
            match ship.think {
                Some(think) => Some(self.routines[think.0].run(me, storage, bullet_system, earth, dt).unwrap()),
                None => None,
            }
        };
        let ship = storage.get_mut(me).unwrap();
        ship.think = new_think;
    }
}

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
                RenderInfo {},
                array_vec![_ => Engine::new(vec2(0.0f32, 1.0f32), 1, 5.0f32, 0)],
                array_vec![_ => Gun::new(vec2(0.0f32, 0.0f32), Bullet::tester_bullet, Duration::from_millis(300), vec2(0.0f32, 1.0f32))],
            ),
        }
    }

    pub fn turret_ship() -> Self {
        TemplateTableEntry {
            name : "Turret enemy".to_owned(),
            prefab : Ship::new(
                Core::new(3, 100.0f32, CollisionModelIndex::Player, Team::Hive),
                Some(RoutineId(0)),
                RenderInfo {},
                array_vec![],
                array_vec![_ => Gun::new(vec2(0.0f32, 0.0f32), Bullet::laser_ball, Duration::from_millis(400), vec2(0.0f32, 1.0f32))],
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
  
    // TODO needs fixing for some more consistent logic
    pub fn think(&mut self, bullet_system : &mut BulletSystem, dt : Duration) {
        use std_ext::*;

        for i in 0..self.mem.len() {
            let alive =
                match self.mem.get(i) {
                    Some(ship) => ship.core.is_alive(),
                    None => continue,
                }
            ;
            if alive {
                //(elem.think)(elem, &extract, bullet_system, &self.earth, dt);
                self.ai_machine.think_for(
                    i,
                    &mut self.mem,
                    bullet_system,
                    &self.earth,
                    dt,
                );
            }
        }
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
    pub fn iter(&self) -> impl Iterator<Item = &Ship> {
        self.mem.iter().map(|(_, x)| x)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Ship> {
        self.mem.iter_mut().map(|(_, x)| x)
    }
}
