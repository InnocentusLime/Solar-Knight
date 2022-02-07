use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use std::ops::AddAssign;
use std::collections::HashMap;

#[derive(Default, Component)]
pub struct CollisionMarker;

pub trait DamageInflictorComponent : Component + Copy {
    type Output : Default + Component + AddAssign;

    fn compute_damage(&self) -> Self::Output;
}

// TODO take collisions into account too
pub fn collision_daemon<Inflictor : DamageInflictorComponent>(
    inflictor_entities : Query<(Entity, &Inflictor)>,
    mut inflictors : Local<HashMap<Entity, (bool, Inflictor)>>,
    mut victims : Local<HashMap<Entity, Inflictor::Output>>,
    mut events : EventReader<IntersectionEvent>,
    mut commands : Commands,
) 
{
    inflictors.clear();
    inflictor_entities.iter()
    .for_each(|(entity, x)| { inflictors.insert(entity, (false, *x)); });

    for (body_a, body_b) in 
        events.iter()
        .map(|x| (x.collider1.entity(), x.collider2.entity())) 
    {
        if let Some((collided, inflictor)) = inflictors.get_mut(&body_b)
        {
            if !*collided {
                *collided = true;
                commands.entity(body_b)
                .insert(CollisionMarker::default());
            }

            *victims.entry(body_a).or_insert(Inflictor::Output::default()) += 
                inflictor.compute_damage()
            ;
        }
        
        if let Some((collided, inflictor)) = inflictors.get_mut(&body_a)
        {
            if !*collided {
                *collided = true;
                commands.entity(body_a)
                .insert(CollisionMarker::default());
            }

            *victims.entry(body_b).or_insert(Inflictor::Output::default()) += 
                inflictor.compute_damage()
            ;
        }
    }

    victims.drain().for_each(|(entity, result)| { commands.entity(entity).insert(result); });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemLabel)]
pub struct CollisionDaemon;

pub struct CollisionDaemonPlugin<Inflictor : DamageInflictorComponent>(std::marker::PhantomData<fn(&mut Inflictor)>);

impl<Inflictor> CollisionDaemonPlugin<Inflictor> 
where
    Inflictor : DamageInflictorComponent
{
    pub fn new() -> Self { CollisionDaemonPlugin(std::marker::PhantomData) }
}

impl<Inflictor> Plugin for CollisionDaemonPlugin<Inflictor> 
where
    Inflictor : DamageInflictorComponent
{
    fn build(&self, app : &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate, 
            collision_daemon::<Inflictor>
            .label(CollisionDaemon)
        );
    }
}
