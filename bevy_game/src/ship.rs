use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::layer_system::{ Layer, LayerComponent };

// Resources essential for ships
pub struct ShipResources {
    environment_friction : f32,
    player_texture : Handle<Image>,
    basic_enemy_texture : Handle<Image>,
}

impl ShipResources {
    // TODO make it a system
    fn load_routine(
        world : &mut World,
    ) {
        if !world.contains_resource::<Self>() {
            let asset_server = world.get_resource::<AssetServer>().unwrap();
            let this = 
                ShipResources {
                    environment_friction : 0.53f32,
                    player_texture : asset_server.load("textures/player_ship.png"),
                    basic_enemy_texture : asset_server.load("textures/enemy_ship.png"),
                }
            ;

            world.insert_resource(this);
        }
    }
}

#[derive(Component)]
pub struct ShipTag;

#[derive(Bundle)]
pub struct ShipBundle {
    name : Name,
    tag : ShipTag,
    sync : ColliderPositionSync,
    #[bundle]
    sprite_bundle : SpriteBundle,
    #[bundle]
    collider_bundle : ColliderBundle,
    #[bundle]
    rigid_body_bundle : RigidBodyBundle,
    layer : LayerComponent,
}

impl ShipBundle {
    pub fn test_ship(
        ship_resources : &ShipResources,
    ) -> Self {
        const SHIP_LAYER_OFFSET : f32 = 2.0f32;

        let mut rigid_body = RigidBodyBundle::default();
        rigid_body.mass_properties.local_mprops.inv_mass = 1.0f32 / 4.0f32;
        rigid_body.mass_properties.flags |= RigidBodyMassPropsFlags::ROTATION_LOCKED;

        ShipBundle {
            name : Name::new("test ship"),
            tag : ShipTag,
            sync : ColliderPositionSync::Discrete,
            sprite_bundle : SpriteBundle {
                transform : 
                    Transform::from_xyz(0.0f32, 0.0f32, SHIP_LAYER_OFFSET)
                    .with_scale(Vec3::new(0.065f32, 0.065f32, 1.0f32))
                ,
                texture : ship_resources.player_texture.clone(),
                sprite : Sprite {
                    custom_size : Some(Vec2::new(2.0f32, 2.0f32)),
                    ..Default::default()
                },
                ..Default::default()
            },
            collider_bundle : ColliderBundle {
                shape : ColliderShape::ball(0.065f32).into(),
                ..ColliderBundle::default()
            },
            rigid_body_bundle : rigid_body,
            layer : LayerComponent {
                layer : Layer::ShipLayer,
                internal_offset : 0.0f32,
            },
        }
    }
}

fn space_friction_system(
    ship_reses : Res<ShipResources>,
    mut query : Query<(&mut RigidBodyForcesComponent, &mut RigidBodyVelocityComponent), With<ShipTag>>,
) {
    for (mut force_info, velocity) in query.iter_mut() {
        let v = velocity.linvel;
        force_info.force -= (ship_reses.environment_friction * v.magnitude()) * v;
    }
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app : &mut App) {
        ShipResources::load_routine(&mut app.world);
        app.add_system(space_friction_system);
    }
}
