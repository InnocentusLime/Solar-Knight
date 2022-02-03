use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use std::collections::HashMap;

pub trait CollisionVictimData : Component + Copy {
    type Input : Copy;

    fn take_damage(&mut self, input : Self::Input);
}

pub trait CollisionInflictorData : Component + Copy {
    type Output : Copy;

    fn successful_impact(&mut self);
    fn compute_damage(&self) -> Self::Output;
}

// TODO take collisions into account too
pub fn collision_daemon<InflictorData, VictimData>(
    mut victim_entities : Query<(Entity, &mut VictimData)>,
    mut inflictor_entities : Query<(Entity, &mut InflictorData)>,
    mut inflictors : Local<HashMap<Entity, InflictorData>>,
    mut victims : Local<HashMap<Entity, VictimData>>,
    mut events : EventReader<IntersectionEvent>,
) 
where
    InflictorData : CollisionInflictorData,
    VictimData : CollisionVictimData<Input = InflictorData::Output>,
{
    victims.clear();
    victim_entities.iter()
    .for_each(|(entity, x)| { victims.insert(entity, *x); });

    inflictors.clear();
    inflictor_entities.iter()
    .for_each(|(entity, x)| { inflictors.insert(entity, *x); });

    for (body_a, body_b) in 
        events.iter()
        .map(|x| (x.collider1.entity(), x.collider2.entity())) 
    {
        if let 
            (Some(victim), Some(inflictor)) = 
            (victims.get_mut(&body_a), inflictors.get_mut(&body_b)) 
        {
            inflictor.successful_impact();
            victim.take_damage(inflictor.compute_damage());
        }
        
        if let 
            (Some(victim), Some(inflictor)) = 
            (victims.get_mut(&body_b), inflictors.get_mut(&body_a)) 
        {
            inflictor.successful_impact();
            victim.take_damage(inflictor.compute_damage());
        }
    }

    victim_entities.iter_mut().for_each(|(entity, mut x)| *x = victims[&entity]);
    inflictor_entities.iter_mut().for_each(|(entity, mut x)| *x = inflictors[&entity]);
}

pub struct CollisionDaemonPlugin<InflictorData, VictimData>(std::marker::PhantomData<fn(&mut InflictorData, &mut VictimData)>);

impl<InflictorData, VictimData> Plugin for CollisionDaemonPlugin<InflictorData, VictimData> 
where
    InflictorData : CollisionInflictorData,
    VictimData : CollisionVictimData<Input = InflictorData::Output>,
{
    fn build(&self, app : &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate, 
            collision_daemon::<InflictorData, VictimData>
        );
    }
}
