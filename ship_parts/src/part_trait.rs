use std::time::Duration;

use crate::core::Core;

pub trait ShipPart {
    fn update(&mut self, core : &mut Core, dt : Duration);
}
