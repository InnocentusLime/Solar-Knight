use std::time::Duration;

use slab::Slab;
use glium::VertexBuffer;
use tinyvec::ArrayVec;
use tinyvec::array_vec;
use serde::{ Serialize, Deserialize };
use cgmath::{ Point2, Matrix4, EuclideanSpace, InnerSpace, vec2, abs_diff_ne, abs_diff_eq };

use cgmath_ext::rotate_vector_ox;

use crate::gun::BulletSystem;
use crate::storage_traits::Battlefield;
use crate::constants::VECTOR_NORMALIZATION_RANGE;

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
        battlefield : &Battlefield,
    ) -> Point2<f32> {
        match self {
            Target::Earth => battlefield.earth.pos(),
            Target::Ship(ship_id) => battlefield.get(ship_id).unwrap().core.pos,
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
        storage : &mut Battlefield, 
        bullet_system : &mut crate::gun::BulletSystem,
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
                let target = target.get_pos(storage);
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
                let target = target.get_pos(storage);
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
                let target = target.get_pos(storage);
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
                bullet_system.shoot_from_gun(storage, me, gun);
                // TODO process the failure
                Ok(ExecutionControl::GoTo(next))
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
        storage : &mut Battlefield, 
        bullet_system : &mut crate::gun::BulletSystem,
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
        storage : &mut Battlefield, 
        bullet_system : &mut crate::gun::BulletSystem,
        dt : Duration,
    ) {
        let ship = storage.get_mut(me).unwrap();
        let new_think = {
            match ship.think {
                Some(think) => Some(self.routines[think.0].run(me, storage, bullet_system, dt).unwrap()),
                None => None,
            }
        };
        let ship = storage.get_mut(me).unwrap();
        ship.think = new_think;
    }
    
    // TODO needs fixing for some more consistent logic
    pub fn update(&self, battlefield : &mut Battlefield, bullet_system : &mut BulletSystem, dt : Duration) {
        use std_ext::*;

        for i in 0..battlefield.len() {
            let alive =
                match battlefield.get(i) {
                    Some(ship) => ship.core.is_alive(),
                    None => continue,
                }
            ;
            if alive {
                self.think_for(
                    i,
                    battlefield,
                    bullet_system,
                    dt,
                );
            }
        }
    }
}

