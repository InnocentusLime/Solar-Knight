use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use crate::collision_daemon::CollisionFilterComponent;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Component, Inspectable)]
pub enum TeamComponent {
    Earth,
    Hive,
    Neutral,
}

impl CollisionFilterComponent for TeamComponent {
    fn can_collide(&self, other : &Self) -> bool {
        *self != *other
    }
}
