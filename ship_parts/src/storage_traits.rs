use std::any::{ Any, TypeId };
use std::time::Duration;

use crate::earth::Earth;
use crate::core::{ Core, Team };
use crate::engine::Engine;
use crate::gun::{ BulletSystem, TargetSystem, Gun };

use std_ext::ExtractResultMut;
use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;
use crate::constants::VECTOR_NORMALIZATION_RANGE;
use cgmath_ext::rotate_vector_ox;

use glium::VertexBuffer;
use tinyvec::ArrayVec;
use cgmath::{ Point2, Matrix4, EuclideanSpace, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

pub static mut FRICTION_KOEFF : f32 = 0.5f32;

#[derive(Clone, Copy, Debug)]
pub struct RoutineId(pub usize);
#[derive(Clone, Copy, Debug)]
pub struct CommandId(pub usize);

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Earth,
    Ship(usize),
// Forgotten for now
//  Current,
}

impl Target {
    // TODO all those ship_id checks should
    // go here.
    fn get_pos(
        self,
        others : &std_ext::ExtractResultMut<Ship>, 
        earth : &Earth,
    ) -> Point2<f32> {
        match self {
            Target::Earth => earth.pos(),
            Target::Ship(ship_id) => others[ship_id].core.pos,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ExecutionControl {
    GoTo(CommandId),
    Done(RoutineId),
}

#[derive(Clone, Copy, Debug)]
pub enum CommandError {
    TargetOutOfRange,
    TriedToTargetSelf,
    GunNotPresent,
}

// Ai routine is a block of commands which decide ships new state and
// then says which ai routine should be executed next frame
#[derive(Clone, Copy, Debug)]
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
}

impl AiCommand {
    pub fn run(
        self, 
        me : &mut Ship,
        others : &std_ext::ExtractResultMut<Ship>, 
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
                let target = target.get_pos(others, earth);
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
                let target = target.get_pos(others, earth);
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
                let target = target.get_pos(others, earth);
                let dir_vec = (target - me.core.pos).normalize();
                let dir_vec = dir_vec.normalize();
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
                match me.guns.get_mut(gun) {
                    Some(gun) => {
                        gun.shoot(&me.core) 
                        .map_or(
                            (),
                            |x| bullet_system.spawn(x)
                        );
                        Ok(ExecutionControl::GoTo(next))
                    },
                    None => Err(CommandError::GunNotPresent),
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
        me : &mut Ship,
        others : &std_ext::ExtractResultMut<Ship>, 
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
                        others, 
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
    pub fn think_for(
        &self,
        me : &mut Ship,
        others : &std_ext::ExtractResultMut<Ship>, 
        bullet_system : &mut crate::gun::BulletSystem,
        earth : &Earth,
        dt : Duration,
    ) {
        match me.think {
            Some(think) => me.think = Some(self.routines[think.0].run(me, others, bullet_system, earth, dt).unwrap()),
            None => (),
        }
    }
}

// TODO probably should place the gun limit into
// a separate constant
#[derive(Clone, Copy)]
pub struct Ship {
    pub render : fn(&Self, &mut SpriteDataWriter),
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
    pub ai_machine : AiMachine,
}

impl Battlefield {
    pub fn new() -> Battlefield {
        Battlefield {
            mem : Vec::new(),
            earth : Earth::new(),
            ai_machine : AiMachine::new(),
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
                let acceleration = c.core.force / c.core.mass;
                c.core.pos += dt.as_secs_f32() * c.core.velocity + dt.as_secs_f32().powi(2) * acceleration / 2.0f32;
                c.core.velocity += dt.as_secs_f32() * acceleration;
            }
        );

        self.mem.retain(|x| x.core.is_alive() || x.core.team() == Team::Earth);
    }
            
    pub fn think(&mut self, bullet_system : &mut BulletSystem, dt : Duration) {
        use std_ext::*;

        for i in 0..self.mem.len() {
            let (extract, elem) = self.mem.as_mut_slice().extract_mut(i);

            if elem.core.is_alive() {
                //(elem.think)(elem, &extract, bullet_system, &self.earth, dt);
                self.ai_machine.think_for(
                    elem,
                    &extract,
                    bullet_system,
                    &self.earth,
                    dt,
                );
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
