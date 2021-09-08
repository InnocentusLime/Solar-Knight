use nalgebra::{ Unit, Complex, UnitComplex, Vector2 };
//use serde::{ Serialize, Deserialize };
use tinyvec::ArrayVec;

use ship_transform::Transform;
use physics::PhysicsData;
use systems_core::{ get_component, get_component_mut, ComponentAccess, Storage, MutationObserver, Observation };

#[derive(Clone, Copy, Debug)]
pub struct Engine {
    pub direction : UnitComplex<f32>,
    max_lvl : u16,
    force_mul : f32,
    current_lvl : u16
}

pub const VECTOR_NORMALIZATION_RANGE : f32 = 0.0001f32;
impl Engine {
    pub fn new(
        direction : UnitComplex<f32>,
        max_lvl : u16,
        force_mul : f32,
        start_lvl : u16,
    ) -> Self {
        assert!(start_lvl <= max_lvl);

        Engine {
            direction,
            max_lvl,
            force_mul,
            current_lvl : start_lvl,
        }
    }
            
    #[inline]
    pub fn increase_speed(&mut self) {
        self.current_lvl = (self.current_lvl + 1).min(self.max_lvl);
    }

    #[inline]
    pub fn decrease_speed(&mut self) {
        self.current_lvl = self.current_lvl.saturating_sub(1); 
    }

    #[inline]
    pub fn level(&self) -> u16 { self.current_lvl }

    #[inline]
    pub fn set_level(&mut self, level : u16) {
        self.current_lvl = level.min(self.max_lvl);
    }
}

impl Default for Engine {
    fn default() -> Engine {
        Engine::new(Unit::new_normalize(Complex::new(0.0f32, 1.0f32)), 0, 0.0f32, 0)
    }
}

#[macro_export]
macro_rules! exponential_decrease_curve {
    ($base:expr) => { 
        |t : f32, dt : f32, curr : f32, end : f32 | { 
            let frac_div = $base.powf(t) - 1.0f32;
            if frac_div < std::f32::EPSILON { end }
            else {
                let frac = ($base.powf((t - dt).max(0.0f32)) - 1.0f32) / ($base.powf(t) - 1.0f32);
                ((curr - end) * frac + end) 
            }
        }
    }
}

pub const ENGINE_LIMIT : usize = 5;
#[derive(Clone, Copy, Debug)]
pub struct Engines {
    pub engines : ArrayVec<[Engine; ENGINE_LIMIT]>,
}

pub struct EngineSystem;

impl EngineSystem {
    pub fn new() -> Self {
        EngineSystem
    }

    fn engine_force(engine : &Engine, dir : UnitComplex<f32>) -> Vector2<f32> {
        let direction = dir * engine.direction;
        (engine.force_mul * engine.current_lvl as f32) * (direction * Vector2::new(1.0f32, 0.0f32))
    }

    pub fn update<Host, Observer>(
        &mut self, 
        observation : &mut Observation<Observer, Host>, 
    ) 
    where
        Host : Storage,
        Host::Object : ComponentAccess<PhysicsData> + ComponentAccess<Engines> + ComponentAccess<Transform>,
        Observer : MutationObserver<Host>,
    {
        observation.mutate_each(
            |obj, _| {
                let dir = get_component::<Transform, _>(obj).transform.rotation;
                let force : Vector2<f32> = 
                    get_component::<Engines, _>(obj).engines.iter()
                    .map(|e| Self::engine_force(e, dir))
                    .sum()
                ;
                get_component_mut::<PhysicsData, _>(obj).force += force;
            }
        )
    }
}
