use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use std::ops::AddAssign;
use std::collections::HashMap;

// TODO sparse set storage
#[derive(Default, Component)]
pub struct CollisionMarker;

pub trait CollisionInflictorComponent : Component + Copy {
    // tip: it's probably better to use sparse set storage
    // method for the output component
    type Output : Default + Component + AddAssign;

    fn compute_effect(&self) -> Self::Output;
}

/// A component to help filtering off the collisions between
/// entities. 
/// Note that in order to not create strange situations it is
/// advised that the implementation of this trait is symmetric
/// As in, `a.can_collide(b) == b.can_collide(a)`
pub trait CollisionFilterComponent : Component + Copy {
    fn can_collide(&self, other : &Self) -> bool;
}

// TODO take collisions into account too
pub fn collision_daemon<Inflictor : CollisionInflictorComponent, Filter : CollisionFilterComponent>(
    inflictor_entities : Query<(Entity, &Inflictor)>,
    filters : Query<(Entity, &Filter)>,
    mut filter_table : Local<HashMap<Entity, Filter>>,
    mut inflictors : Local<HashMap<Entity, (bool, Inflictor)>>,
    mut victims : Local<HashMap<Entity, Inflictor::Output>>,
    mut events : EventReader<IntersectionEvent>,
    mut commands : Commands,
) 
{
    inflictors.clear();
    inflictor_entities.iter()
    .for_each(|(entity, x)| { inflictors.insert(entity, (false, *x)); });

    filter_table.clear();
    filters.iter()
    .for_each(|(entity, x)| { filter_table.insert(entity, *x); });

    for (body_a, body_b) in 
        events.iter()
        .map(|x| (x.collider1.entity(), x.collider2.entity())) 
    {
        match (filter_table.get(&body_a), filter_table.get(&body_b)) {
            (Some(filter_a), Some(filter_b)) if filter_a.can_collide(filter_b) => (),
            _ => continue,
        }

        if let Some((collided, inflictor)) = inflictors.get_mut(&body_b)
        {
            if !*collided {
                *collided = true;
                commands.entity(body_b)
                .insert(CollisionMarker::default());
            }

            *victims.entry(body_a).or_insert(Inflictor::Output::default()) += 
                inflictor.compute_effect()
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
                inflictor.compute_effect()
            ;
        }
    }

    victims.drain().for_each(|(entity, result)| { commands.entity(entity).insert(result); });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemLabel)]
pub struct CollisionDaemon;

pub struct CollisionDaemonPlugin<Inflictor : CollisionInflictorComponent, Filter : CollisionFilterComponent>(std::marker::PhantomData<fn(&mut Inflictor, &Filter)>);

impl<Inflictor, Filter> CollisionDaemonPlugin<Inflictor, Filter> 
where
    Inflictor : CollisionInflictorComponent,
    Filter : CollisionFilterComponent,
{
    pub fn new() -> Self { CollisionDaemonPlugin(std::marker::PhantomData) }
}

impl<Inflictor, Filter> Plugin for CollisionDaemonPlugin<Inflictor, Filter> 
where
    Inflictor : CollisionInflictorComponent,
    Filter : CollisionFilterComponent,
{
    fn build(&self, app : &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate, 
            collision_daemon::<Inflictor, Filter>
            .label(CollisionDaemon)
        );
    }
}
