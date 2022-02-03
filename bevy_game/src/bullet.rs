use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// TODO wire up to collision daemon
use crate::layer_system::{ Layer, LayerComponent };

pub struct BulletResources {
    basic_bullet_texture : Handle<Image>,
}

impl BulletResources {
    fn load_routine(
        world : &mut World,
    ) {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let this = 
            BulletResources {
                basic_bullet_texture : asset_server.load("textures/player_bullet.png"),
            }
        ;

        world.insert_resource(this);
    }
}

#[derive(Clone, Copy, Component)]
pub struct BulletTag;

#[derive(Component)]
pub struct LifetimeTimerComponent(Timer);

pub fn timed_out_entity_system(
    time : Res<Time>,
    mut commands : Commands,
    mut query : Query<(Entity, &mut LifetimeTimerComponent)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        if lifetime.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    name : Name,
    tag : BulletTag,
    sync : ColliderPositionSync,
    #[bundle]
    sprite_bundle : SpriteBundle,
    #[bundle]
    collider_bundle : ColliderBundle,
    #[bundle]
    rigid_body_bundle : RigidBodyBundle,
    layer : LayerComponent,
}

impl BulletBundle {
    pub fn test_bullet(
        bullet_resources : &BulletResources,
        x : f32, y : f32
    ) -> Self {
        let mut rigid_body = RigidBodyBundle::default();
        rigid_body.position.position.translation.vector = vector!(x, y);
        rigid_body.mass_properties.flags |= RigidBodyMassPropsFlags::ROTATION_LOCKED;
        
        BulletBundle {
            name : Name::new("test bullet"),
            tag : BulletTag,
            sync : ColliderPositionSync::Discrete,
            sprite_bundle : SpriteBundle {
                transform : 
                    Transform::identity()
                    .with_scale(Vec3::new(0.021f32, 0.043f32, 1.0f32))
                ,
                texture : bullet_resources.basic_bullet_texture.clone(),
                sprite : Sprite {
                    custom_size : Some(Vec2::new(2.0f32, 2.0f32)),
                    ..Default::default()
                },
                ..Default::default()
            },
            collider_bundle : ColliderBundle {
                collider_type : ColliderType::Sensor.into(),
                shape : ColliderShape::cuboid(0.021f32, 0.043f32).into(),
                flags : ActiveEvents::INTERSECTION_EVENTS.into(),
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

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app : &mut App) {
        BulletResources::load_routine(&mut app.world);
        app
        .add_system(timed_out_entity_system);
    }
}
