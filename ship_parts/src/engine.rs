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

// snappy_engine --- snaps into the picked level
// soft_engine --- gradually increases/decreases towards the picked level
// * requires f : (t, current, end) -> current
#[macro_export]
macro_rules! declare_engine {
    (
        snappy_engine $name:ident { 
            speed_mul : $speed_mul:expr, 
            max_lvl : $max_lvl:expr, 
            direction : ($dir_x:expr, $dir_y:expr), 
        }
    ) => {
        #[derive(Clone, Copy)]
        pub struct $name {
            current_mul : u64,
        }

        impl $name {
            pub const SPEED : f32 = $speed_mul;
            pub const MAX_LVL : u64 = $max_lvl;
            pub const DIRECTION : cgmath::Vector2<f32> = cgmath::vec2($dir_x, $dir_y);

            pub fn new(start : u64) -> Self {
                use cgmath::InnerSpace;
                use $crate::constants::VECTOR_NORMALIZATION_RANGE;

                debug_assert!((Self::DIRECTION.magnitude() - 1.0f32) < VECTOR_NORMALIZATION_RANGE);
                Self {
                    current_mul : start.min(Self::MAX_LVL),
                }
            }

            #[inline]
            pub fn set_speed(&mut self, val : u64) {
                self.current_mul = val.min(Self::MAX_LVL)
            }

            #[inline]
            pub fn get_speed(&self) -> u64 { self.current_mul }

            #[inline]
            pub fn increase_speed(&mut self) {
                self.current_mul = (self.current_mul + 1).min(Self::MAX_LVL);
            }

            #[inline]
            pub fn decrease_speed(&mut self) {
                self.current_mul = self.current_mul.saturating_sub(1); 
            }
            
            /// Rotates the direction vector according to engine's relative
            /// rotation.
            #[inline]
            pub fn map_direction(&self, direction : cgmath::Vector2<f32>) -> cgmath::Vector2<f32> {
                cgmath::vec2(
                    Self::DIRECTION.y * direction.x + Self::DIRECTION.x * direction.y,
                    -Self::DIRECTION.x * direction.x + Self::DIRECTION.y * direction.y,
                )
            }

            #[inline]
            pub fn update(&mut self, core : &mut $crate::core::Core, dt : std::time::Duration) {
                use cgmath::InnerSpace;
                use $crate::constants::VECTOR_NORMALIZATION_RANGE;
                
                debug_assert!((core.direction.magnitude() - 1.0f32).abs() < VECTOR_NORMALIZATION_RANGE);
                core.pos = core.pos + dt.as_secs_f32() * (Self::SPEED * self.current_mul as f32) * self.map_direction(core.direction)
            }
        }
    };
    (
        soft_engine $name:ident { 
            speed_mul : $speed_mul:expr, 
            max_lvl : $max_lvl:expr, 
            direction : ($dir_x:expr, $dir_y:expr),
            one_step_duration : $one_step_duration:expr,
            change_curve : $change_curve:expr,
        }
    ) => {
        /// This is a soft engine, which means it's speed won't
        /// snap into a certain value but instead will slowly reach
        /// towards it.
        #[derive(Clone, Copy)]
        pub struct $name {
            target_mul : u64,
            current_mul : f32,
            time : std::time::Duration,
        }

        impl $name {
            pub const SPEED : f32 = $speed_mul;
            pub const MAX_LVL : u64 = $max_lvl;
            pub const DIRECTION : cgmath::Vector2<f32> = cgmath::vec2($dir_x, $dir_y);
            pub const ONE_STEP_DURATION : std::time::Duration = $one_step_duration;
            pub const CHANGE_CURVE : fn(f32, f32, f32, f32) -> f32 = $change_curve;
            
            pub fn new(start : u64) -> Self {
                use cgmath::InnerSpace;
                use $crate::constants::VECTOR_NORMALIZATION_RANGE;
                
                debug_assert!((Self::DIRECTION.magnitude() - 1.0f32) < VECTOR_NORMALIZATION_RANGE);
                Self {
                    target_mul : start.min(Self::MAX_LVL),
                    current_mul : start.min(Self::MAX_LVL) as f32,
                    time : <std::time::Duration as std_ext::DurationExt>::my_zero(),
                }
            }

            /// Gets the current speed multiplier
            #[inline]
            pub fn get_speed(&self) -> f32 { self.current_mul }
            
            /// This will set engines interior data to the specific
            /// value
            #[inline]
            pub fn snap(&mut self, x : u64) {
                self.target_mul = x.min(Self::MAX_LVL);
                self.current_mul = self.target_mul as f32;
            }
           
            /// Makes the engine speed start increasing
            #[inline]
            pub fn increase_speed(&mut self) {
                self.target_mul = (self.target_mul + 1).min(Self::MAX_LVL);
                self.time = Self::ONE_STEP_DURATION.mul_f32((self.target_mul as f32 - self.current_mul).abs());
            }

            /// Makes the engine speed start decreasing
            #[inline]
            pub fn decrease_speed(&mut self) {
                self.target_mul = self.target_mul.saturating_sub(1); 
                self.time = Self::ONE_STEP_DURATION.mul_f32((self.target_mul as f32 - self.current_mul).abs());
            }

            /// Check if the engine is in processing of changing its speed
            #[inline]
            pub fn is_changing(&self) -> bool { 
                use std_ext::*;

                !self.time.my_is_zero() 
            }

            /// Rotates the direction vector according to engine's relative
            /// rotation.
            #[inline]
            pub fn map_direction(&self, direction : cgmath::Vector2<f32>) -> cgmath::Vector2<f32> {
                cgmath::vec2(
                    Self::DIRECTION.y * direction.x + Self::DIRECTION.x * direction.y,
                    -Self::DIRECTION.x * direction.x + Self::DIRECTION.y * direction.y,
                )
            }

            /// Applies the engine power on a point.
            /// `direction` encodes the direction which the ship is pointing in.
            #[inline]
            pub fn update(&mut self, core : &mut $crate::core::Core, dt : std::time::Duration) {
                use cgmath::{ InnerSpace, vec2 };
                
                use std_ext::*;
                use $crate::constants::VECTOR_NORMALIZATION_RANGE;
                
                if !self.time.my_is_zero() {
                    self.current_mul = 
                        Self::CHANGE_CURVE(
                            self.time.as_secs_f32(), 
                            dt.as_secs_f32(), 
                            self.current_mul, 
                            self.target_mul as f32
                        )
                    ;
                    self.time = self.time.my_saturating_sub(dt);
                } else { self.current_mul = self.target_mul as f32; }
                
                debug_assert!((core.direction.magnitude() - 1.0f32).abs() < VECTOR_NORMALIZATION_RANGE);
                core.pos = core.pos + dt.as_secs_f32() * (Self::SPEED * self.current_mul) * self.map_direction(core.direction)
            }
        }
    };
    (
        directed_snappy_engine $name:ident { 
            speed_mul : $speed_mul:expr, 
            max_lvl : $max_lvl:expr, 
        }
    ) => {
        #[derive(Clone, Copy)]
        pub struct $name {
            current_mul : u64,
            pub direction : cgmath::Vector2<f32>,
        }

        impl $name {
            pub const SPEED : f32 = $speed_mul;
            pub const MAX_LVL : u64 = $max_lvl;

            pub fn new(start : u64) -> Self {
                Self {
                    current_mul : start.min(Self::MAX_LVL),
                    direction : cgmath::vec2(0.0f32, 1.0f32),
                }
            }

            #[inline]
            pub fn set_speed(&mut self, val : u64) {
                self.current_mul = val.min(Self::MAX_LVL)
            }

            #[inline]
            pub fn get_speed(&self) -> u64 { self.current_mul }

            #[inline]
            pub fn increase_speed(&mut self) {
                self.current_mul = (self.current_mul + 1).min(Self::MAX_LVL);
            }

            #[inline]
            pub fn decrease_speed(&mut self) {
                self.current_mul = self.current_mul.saturating_sub(1); 
            }
            
            #[inline]
            pub fn update(&mut self, core : &mut $crate::core::Core, dt : std::time::Duration) {
                use cgmath::{ InnerSpace };
                use $crate::constants::VECTOR_NORMALIZATION_RANGE;
                
                debug_assert!((self.direction.magnitude() - 1.0f32).abs() < VECTOR_NORMALIZATION_RANGE);
                core.pos = core.pos + dt.as_secs_f32() * (Self::SPEED * self.current_mul as f32) * self.direction
            }
        }
    };
    (
        directed_soft_engine $name:ident { 
            speed_mul : $speed_mul:expr, 
            max_lvl : $max_lvl:expr, 
            one_step_duration : $one_step_duration:expr,
            change_curve : $change_curve:expr,
        }
    ) => {
        /// This is a soft engine, which means it's speed won't
        /// snap into a certain value but instead will slowly reach
        /// towards it.
        #[derive(Clone, Copy)]
        pub struct $name {
            target_mul : u64,
            current_mul : f32,
            time : std::time::Duration,
            pub direction : cgmath::Vector2<f32>,
        }

        impl $name {
            pub const SPEED : f32 = $speed_mul;
            pub const MAX_LVL : u64 = $max_lvl;
            pub const ONE_STEP_DURATION : std::time::Duration = $one_step_duration;
            pub const CHANGE_CURVE : fn(f32, f32, f32, f32) -> f32 = $change_curve;
            
            pub fn new(start : u64) -> Self {
                Self {
                    target_mul : start.min(Self::MAX_LVL),
                    current_mul : start.min(Self::MAX_LVL) as f32,
                    time : <std::time::Duration as std_ext::DurationExt>::my_zero(),
                    direction : cgmath::vec2(0.0f32, 1.0f32),
                }
            }

            /// Gets the current speed multiplier
            #[inline]
            pub fn get_speed(&self) -> f32 { self.current_mul }
            
            /// This will set engines interior data to the specific
            /// value
            #[inline]
            pub fn snap(&mut self, x : u64) {
                self.target_mul = x.min(Self::MAX_LVL);
                self.current_mul = self.target_mul as f32;
            }
           
            /// Makes the engine speed start increasing
            #[inline]
            pub fn increase_speed(&mut self) {
                self.target_mul = (self.target_mul + 1).min(Self::MAX_LVL);
                self.time = Self::ONE_STEP_DURATION.mul_f32((self.target_mul as f32 - self.current_mul).abs());
            }

            /// Makes the engine speed start decreasing
            #[inline]
            pub fn decrease_speed(&mut self) {
                self.target_mul = self.target_mul.saturating_sub(1); 
                self.time = Self::ONE_STEP_DURATION.mul_f32((self.target_mul as f32 - self.current_mul).abs());
            }

            /// Check if the engine is in processing of changing its speed
            #[inline]
            pub fn is_changing(&self) -> bool { 
                use std_ext::*;

                !self.time.my_is_zero() 
            }

            /// Applies the engine power on a point.
            /// `direction` encodes the direction which the ship is pointing in.
            #[inline]
            pub fn update(&mut self, core : &mut $crate::core::Core, dt : std::time::Duration) {
                use cgmath::{ InnerSpace, vec2 };
                use std_ext::*;
                use $crate::constants::VECTOR_NORMALIZATION_RANGE;
                
                if !self.time.my_is_zero() {
                    self.current_mul = 
                        Self::CHANGE_CURVE(
                            self.time.as_secs_f32(), 
                            dt.as_secs_f32(), 
                            self.current_mul, 
                            self.target_mul as f32
                        )
                    ;
                    self.time = self.time.my_saturating_sub(dt);
                } else { self.current_mul = self.target_mul as f32; }
                
                debug_assert!((self.direction.magnitude() - 1.0f32).abs() < VECTOR_NORMALIZATION_RANGE);
                core.pos = core.pos + dt.as_secs_f32() * (Self::SPEED * self.current_mul) * self.direction
            }
        }
    };
}

//declare_engines!(engine Test { speed_mul : 1.0f32, max_lvl : 4, direction : (0.0f32, 1.0f32) });
/*
declare_engine!(
    directed_soft_engine TestSoftEngine {
        speed_mul : 0.4f32,
        max_lvl : 3,
        one_step_duration : std::time::Duration::from_nanos(20),
        change_curve : exponential_decrease_curve!(2.0f32),
    }
);
*/
