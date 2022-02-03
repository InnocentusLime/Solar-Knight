use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_inspector_egui::Inspectable;

// TODO wire up to collision daemon
use crate::layer_system::{ Layer, LayerComponent };

#[derive(Inspectable)]
pub struct ShipConfig {
    environment_friction : f32,
}

impl ShipConfig {
    pub fn new() -> Self {
        ShipConfig {
            environment_friction : 0.53f32,
        }
    }
}

// Resources essential for ships
pub struct ShipResources {
    player_texture : Handle<Image>,
    basic_enemy_texture : Handle<Image>,
}

impl ShipResources {
    // TODO make it a system
    fn load_routine(
        world : &mut World,
    ) {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let this = 
            ShipResources {
                player_texture : asset_server.load("textures/player_ship.png"),
                basic_enemy_texture : asset_server.load("textures/enemy_ship.png"),
            }
        ;

        world.insert_resource(this);
    }
}

#[derive(Clone, Copy, Component)]
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
        let mut rigid_body = RigidBodyBundle::default();
        rigid_body.mass_properties.local_mprops.inv_mass = 1.0f32 / 4.0f32;
        rigid_body.mass_properties.flags |= RigidBodyMassPropsFlags::ROTATION_LOCKED;

        ShipBundle {
            name : Name::new("test ship"),
            tag : ShipTag,
            sync : ColliderPositionSync::Discrete,
            sprite_bundle : SpriteBundle {
                transform : 
                    Transform::identity()
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
    ship_reses : Res<ShipConfig>,
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
        app
        .add_system(space_friction_system)
        .insert_resource(ShipConfig::new());
    }
}
