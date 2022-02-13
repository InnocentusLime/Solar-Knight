use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use std::ops::AddAssign;

#[derive(Clone, Copy, Debug, Component, Inspectable)]
pub struct DamageComponent {
    pub plasma_damage : u32,
}

impl Default for DamageComponent {
    fn default() -> Self {
        DamageComponent {
            plasma_damage : 0,
        }
    }
}

impl AddAssign for DamageComponent {
    fn add_assign(&mut self, rhs : Self) {
        self.plasma_damage += rhs.plasma_damage;
    }
}

#[derive(Clone, Copy, Debug, Component, Inspectable)]
pub struct HealthComponent {
    pub health : u32,
}

impl HealthComponent {
    pub fn take_damage(&mut self, dmg : DamageComponent) {
        self.health = self.health.saturating_sub(dmg.plasma_damage);
    }
}
