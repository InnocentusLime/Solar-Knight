use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy::ecs::system::EntityCommands;

use crate::team::TeamComponent;
use crate::health::DamageComponent;
use crate::layer_system::{ Layer, LayerComponent };
use crate::collision_daemon::{ CollisionInflictorComponent, CollisionMarker };

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

#[derive(Clone, Copy, Debug, Component, Inspectable)]
pub struct BulletAttributes {
    damage : DamageComponent,
//  speed
}

impl BulletAttributes {
    pub fn new(damage : DamageComponent) -> Self {
        BulletAttributes {
            damage,
        }
    }
}

impl CollisionInflictorComponent for BulletAttributes {
    type Output = DamageComponent;

    fn compute_effect(&self) -> Self::Output { self.damage }
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

#[derive(Component)]
pub struct DespawnOnImpact;

pub fn despawn_on_impact_system(
    mut commands : Commands,
    query : Query<Entity, (With<BulletTag>, With<DespawnOnImpact>, With<CollisionMarker>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Bundle)]
struct BaseBulletBundle {
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
    attributes : BulletAttributes,
    team : TeamComponent,
}

pub trait BulletCommands<'w, 's> {
    fn spawn_test_bullet<'a>(
        &'a mut self,
        bullet_resources : &BulletResources,
        x : f32, y : f32,
        team : TeamComponent,
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> BulletCommands<'w, 's> for Commands<'w, 's> {
    fn spawn_test_bullet<'a>(
        &'a mut self,
        bullet_resources : &BulletResources,
        x : f32, y : f32,
        team : TeamComponent,
    ) -> EntityCommands<'w, 's, 'a> {
        let mut rigid_body = RigidBodyBundle::default();
        rigid_body.position.position.translation.vector = vector!(x, y);
        rigid_body.mass_properties.flags |= RigidBodyMassPropsFlags::ROTATION_LOCKED;
       
        let mut commands =
            self.spawn_bundle(BaseBulletBundle {
                team,
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
                attributes : BulletAttributes::new(
                    DamageComponent { plasma_damage : 1, },
                ),
            })
        ;
        commands.insert(DespawnOnImpact);

        commands
    }
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app : &mut App) {
        BulletResources::load_routine(&mut app.world);
        app
        .add_system(timed_out_entity_system)
        .add_system_to_stage(
            CoreStage::Last, 
            despawn_on_impact_system
            //.after(CollisionDaemon)
        );
    }
}
