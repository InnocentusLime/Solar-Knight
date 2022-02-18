use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::prelude::nalgebra::{ Unit, Complex };
use bevy::ecs::system::EntityCommands;

use crate::team::TeamComponent;
use crate::mouse_pos::get_mouse_position;
use crate::bullet::BulletCommands;
use crate::ship::{ ShipCommands, ShipResources };

#[derive(Component)]
pub struct PlayerShipTag;

pub trait PlayerCommands<'w, 's> {
    fn spawn_player<'a>(
        &'a mut self,
        ship_resources : &ShipResources,
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> PlayerCommands<'w, 's> for Commands<'w, 's> {
    fn spawn_player<'a>(
        &'a mut self,
        ship_resources : &ShipResources,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut commands = self.spawn_test_ship(ship_resources, TeamComponent::Earth);

        commands
        .insert(PlayerShipTag)
        .insert(Name::new("Player"));

        commands
    }
}

fn player_controls(
    wnds : Res<Windows>,
    mouse_button_input : Res<Input<MouseButton>>,
    camera_query : Query<&Transform, With<Camera>>,
//    mut commands : Commands,
    mut player_query : Query<(&mut RigidBodyPositionComponent, &mut RigidBodyForcesComponent), With<PlayerShipTag>>
) {
    const PLAYER_FORCE_MAGNITUDE : f32 = 10.0f32;
    let mouse_pos = 
        match get_mouse_position(&wnds, camera_query.single()) {
            Some(x) => x.to_array(),
            None => return,
        }
    ;
    let (mut player_trans, mut player_forces) = player_query.single_mut();

    let player_pos = player_trans.position.translation.vector;
    let mouse_pos = vector!(mouse_pos[0], mouse_pos[1]);

    let dir_vec = (mouse_pos - player_pos).normalize();

    // TODO nalgebra -> bevy conversion lib
    if mouse_button_input.pressed(MouseButton::Right) {
        player_forces.force += PLAYER_FORCE_MAGNITUDE * dir_vec;
    }
    player_trans.position.rotation = Unit::new_normalize(Complex::new(dir_vec.x, dir_vec.y));
}

pub struct PlayerShipPlugin;

impl Plugin for PlayerShipPlugin {
    fn build(&self, app : &mut App) {
        app.add_system(player_controls);
    }
}
