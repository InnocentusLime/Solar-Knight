use std::time::Duration;
use std::collections::{ HashMap, HashSet };

use slab::Slab;
use tinyvec::ArrayVec;
use nalgebra::{ Isometry2, Vector2, UnitComplex, Matrix4 };

use teams::Team;
use hp_system::HpInfo;
use square_map::{ SquareMap, SquareMapNode };
use collision_check::{ CollisionSystem, CollisionInfo };
use systems_core::{ ComponentAccess, get_component, get_component_mut, Observation, MutationObserver, DeletionObserver, Storage, SystemAccess, get_system };
use ship_transform::Transform;
use sys_api::graphics_init::PLAYER_BULLET_LIMIT;
use std_ext::duration_ext::*;
use sys_api::graphics_init::SCREEN_WIDTH;

pub const VECTOR_NORMALIZATION_RANGE : f32 = 0.0001f32;
pub const TESTER_BULLET_SIZE : (f32, f32) = (0.06f32, 0.09f32);

// TODO macrofied-bullet construction tool

/// The bullet kind. Can hold data
#[derive(Clone, Copy, Debug)]
pub enum BulletKind {
    TestBullet,
    LaserBall,
    SpinningLaser,
    LaserBeam,
    HomingMissle,
}

#[derive(Clone, Copy, Debug)]
pub enum BulletData {
    TestBullet,
    LaserBall,
    SpinningLaser, // Maybe add damange cooldown
    LaserBeam,
    HomingMissle {
        align_timer : Duration,
        target : Option<usize>,
    },
}

// TODO bullets shouldn't be baked into
// the bullet system
#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pub transform : Transform,
    pub kind : BulletData,
    pub lifetime : Duration,
    pub team : Team,
    pub parent : usize,
}

impl Bullet {
    const HOMING_MISSLE_ALIGN_TIME : Duration = Duration::from_millis(500);
    
    #[inline]
    pub fn size(&self) -> (f32, f32) {
        match self.kind {
            BulletData::TestBullet => (0.09f32, 0.06f32),
            BulletData::LaserBall => (0.03f32, 0.03f32),
            BulletData::SpinningLaser { .. } | BulletData::LaserBeam { .. } => (SCREEN_WIDTH / 1.5f32, 0.03f32),
            BulletData::HomingMissle { .. } => (0.09f32, 0.06f32),
        }
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        self.transform.model_mat(self.size())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gun {
    offset : Vector2<f32>,
    bullet_kind : BulletKind,
    recoil : Duration,
    timer : Duration,
    direction : UnitComplex<f32>,
}

impl Gun {
    pub fn new(
        offset : Vector2<f32>,
        bullet_kind : BulletKind,
        recoil : Duration,
        direction : UnitComplex<f32>,
    ) -> Self {
        Gun {
            offset,
            bullet_kind,
            recoil,
            timer : <Duration as DurationExt>::my_zero(),
            direction,
        }
    }

    #[inline]
    pub fn kind(&self) -> BulletKind {
        self.bullet_kind
    }
            
    #[inline]
    pub fn can_shoot(&self) -> bool {
        self.timer.my_is_zero()
    }
            
    fn shoot(&mut self) {
        if self.can_shoot() {
            self.timer = self.recoil;
        }
    }
            
    fn update(&mut self, dt : std::time::Duration) {
        self.timer = self.timer.my_saturating_sub(dt);
    }
}

impl Default for Gun {
    fn default() -> Gun {
        Gun::new(
            Vector2::new(0.0f32, 0.0f32),
            BulletKind::TestBullet,
            <Duration as DurationExt>::my_zero(),
            UnitComplex::new(0.0f32)
        )
    }
}

pub const ENGINE_LIMIT : usize = 5;
#[derive(Clone, Copy, Debug)]
pub struct Guns {
    pub guns : ArrayVec<[Gun; ENGINE_LIMIT]>,
}
    
// TODO bench this versus the UID strategy
pub struct BulletSystem {
    mem : Slab<Bullet>,
    // tracked_ships : ship -> {bullets}
    tracked_ships : HashMap<usize, HashSet<usize>>,
}

impl BulletSystem {
    pub fn new() -> Self {
        BulletSystem {
            mem : Slab::with_capacity(PLAYER_BULLET_LIMIT),
            tracked_ships : HashMap::new(),
        }
    }

    pub fn spawn(&mut self, bullet : Bullet) {
        self.mem.insert(bullet);
    }

    // TODO `parent` exists purely because the laser can't work without
    // it. After bullet rework we can get rid of it and make the API
    // better in general
    pub fn shoot_from_gun_ship<Obj>(
        &mut self,
        obj : &mut Obj,
        parent : usize,
        gun : usize,
    ) 
    where
        Obj : ComponentAccess<Guns> + ComponentAccess<Team> + ComponentAccess<Transform>,
    {
        let (shooter_pos, shooter_dir) = {
            let transform = get_component::<Transform, _>(obj);
            (transform.position(), transform.rotation())
        }; 
        let shooter_team = *get_component::<Team, _>(obj);

        let gun = get_component_mut::<Guns, _>(obj).guns.get_mut(gun).unwrap();

        let bullet_pos = shooter_pos + shooter_dir.transform_vector(&gun.offset);
        let bullet_dir = shooter_dir * gun.direction;

        if !gun.can_shoot() { return; }; gun.shoot();

        match gun.kind() {
            BulletKind::TestBullet => {
                self.spawn(        
                    Bullet {
                        transform : Transform::new(Isometry2 {
                            rotation : bullet_dir,
                            translation : bullet_pos.into(),
                        }),
                        team : shooter_team,
                        kind : BulletData::TestBullet,
                        lifetime : Duration::from_secs(3),
                        parent,
                    }
                )
            },
            BulletKind::LaserBall => {
                self.spawn(        
                    Bullet {
                        transform : Transform::new(Isometry2 {
                            rotation : bullet_dir,
                            translation : bullet_pos.into(),
                        }),
                        team : shooter_team,
                        kind : BulletData::LaserBall,
                        lifetime : Duration::from_secs(3),
                        parent,
                    }
                )
            },
            BulletKind::SpinningLaser => {
                self.spawn(        
                    Bullet {
                        transform : Transform::new_with_origin(
                            Isometry2 {
                                rotation : bullet_dir,
                                translation : bullet_pos.into(),
                            },
                            Vector2::new(SCREEN_WIDTH / 1.5f32, 0.0f32)
                        ),
                        team : shooter_team,
                        kind : BulletData::SpinningLaser,
                        lifetime : Duration::from_secs(1),
                        parent,
                    }
                )
            },
            BulletKind::LaserBeam => {
                self.spawn(        
                    Bullet {
                        transform : Transform::new_with_origin(
                            Isometry2 {
                                rotation : bullet_dir,
                                translation : bullet_pos.into(),
                            },
                            Vector2::new(SCREEN_WIDTH / 1.5f32, 0.0f32)
                        ),
                        team : shooter_team,
                        kind : BulletData::LaserBeam,
                        lifetime : Duration::from_secs(3),
                        parent,
                    }
                )
            },
            BulletKind::HomingMissle => {
                let mut spawn_bullet = 
                |direc|
                    self.spawn(        
                        Bullet {
                            transform : Transform::new(Isometry2 {
                                rotation : direc,
                                translation : bullet_pos.into(),
                            }),
                            team : shooter_team,
                            kind : BulletData::HomingMissle { 
                                target : None,
                                align_timer : Bullet::HOMING_MISSLE_ALIGN_TIME,
                            },
                            lifetime : Duration::from_secs(3),
                            parent,
                        }
                    )
                ;
                spawn_bullet(shooter_dir * UnitComplex::new(std::f32::consts::PI * 0.9f32));
                spawn_bullet(shooter_dir * UnitComplex::new(-std::f32::consts::PI * 0.9f32));
                spawn_bullet(shooter_dir * UnitComplex::new(std::f32::consts::PI * 0.7f32));
                spawn_bullet(shooter_dir * UnitComplex::new(-std::f32::consts::PI * 0.7f32));
            },
        }
    }

    // TODO return an error code
    #[inline(always)]
    pub fn shoot_from_gun<S, Observer>(
        &mut self,
        storage : &mut Observation<Observer, S>,
        parent : usize,
        gun : usize,
    ) 
    where
        S : Storage,
        S::Object : ComponentAccess<Transform> + ComponentAccess<Guns> + ComponentAccess<Team>,
        Observer : MutationObserver<S>,
    {
        storage.mutate(parent, |obj, _| self.shoot_from_gun_ship(obj, parent, gun));
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Bullet> {
        self.mem.iter().map(|(_, x)| x)
    }

    pub fn update_guns<S, Observer>(
        &mut self, 
        c : &mut Observation<Observer, S>, 
        dt : Duration
    )
    where
        S : Storage,
        S::Object : ComponentAccess<Guns>,
        Observer : MutationObserver<S>,
    {
        c.mutate_each(
            |obj, _| 
            get_component_mut::<Guns, _>(obj).guns
            .iter_mut().for_each(|g| g.update(dt))
        )
    }

    // FIXME just iterating over all enemies probably sucks.
    pub fn update_bullets<S, Observer>(
        &mut self, 
        c : &mut Observation<Observer, S>,
        collision : &CollisionSystem,
        dt : Duration
    )
    where
        S : Storage,
        S::Object : 
            ComponentAccess<CollisionInfo> + ComponentAccess<Transform> + 
            ComponentAccess<Team> + ComponentAccess<SquareMapNode> + 
            ComponentAccess<HpInfo>
        ,
        Observer : MutationObserver<S> + SystemAccess<SquareMap>,
    {
        use collision_check::*;
        use std_ext::*;

        let tracked_ships = &mut self.tracked_ships;
        self.mem.iter_mut()
        .for_each(
            |(bullet_id, bullet)| {
                bullet.lifetime = bullet.lifetime.my_saturating_sub(dt);

                // Update bullet data and damage on collision
                match &mut bullet.kind {
                    // TestBullet's move towards with speed 
                    // equal to 4.0.
                    BulletData::TestBullet => {
                        bullet.transform.move_in_direction(4.0f32 * dt.as_secs_f32());
        
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }
            
                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, CollisionModelIndex::BulletTester, &bullet.transform.full_transform())
                                {
                                    get_component_mut::<HpInfo, _>(obj).damage(1);
                                    bullet.lifetime = <Duration as DurationExt>::my_zero();
                                } 
                            }
                        )
                    },        
                    BulletData::LaserBall => {
                        bullet.transform.move_in_direction(0.7f32 * dt.as_secs_f32());
        
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }

                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, CollisionModelIndex::LaserBall, &bullet.transform.full_transform())
                                {
                                    get_component_mut::<HpInfo, _>(obj).damage(1);
                                    bullet.lifetime = <Duration as DurationExt>::my_zero();
                                } 
                            }
                        )
                    },
                    BulletData::SpinningLaser => {
                        let parent_pos = 
                            c.get(bullet.parent)
                            .map(|obj| get_component::<Transform, _>(obj).position())
                            .unwrap()
                        ;
                        bullet.transform.transform.translation = parent_pos.into();
                        bullet.transform.transform.rotation = bullet.transform.rotation() * UnitComplex::new(std::f32::consts::TAU * dt.as_secs_f32());
                        
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }
            
                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, CollisionModelIndex::LaserBeam, &bullet.transform.full_transform())
                                {
                                    get_component_mut::<HpInfo, _>(obj).damage(1);
                                } 
                            }
                        )
                    },
                    BulletData::LaserBeam => {
                        let (parent_pos, parent_direction) = 
                            c.get(bullet.parent)
                            .map(|obj| {
                                let transform = get_component::<Transform, _>(obj);
                                (transform.position(), transform.rotation()) 
                            }).unwrap()
                        ;
                        bullet.transform.transform.translation = parent_pos.into();
                        bullet.transform.transform.rotation = parent_direction;
                        
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }
            
                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, CollisionModelIndex::LaserBeam, &bullet.transform.full_transform())
                                {
                                    get_component_mut::<HpInfo, _>(obj).damage(1);
                                } 
                            }
                        )
                    },
                    BulletData::HomingMissle { target, align_timer } => {
                        use std_ext::*;

                        const HOMING_MISSLE_SEE_RANGE : f32 = 2.0f32;
                        match target {
                            Some(target) => {
                                let target = 
                                    get_component::<Transform, _>(
                                        c.get(*target)
                                        .unwrap()
                                    ).position()
                                ;
                                if align_timer.my_is_zero() {
                                    bullet.transform.point_at(target);
                                } else {
                                    *align_timer = align_timer.saturating_sub(dt);
                                    bullet.transform.rotate_towards(target, std::f32::consts::TAU * 0.7f32, dt);
                                }
                            },
                            None => { 
                                let bullet_team = bullet.team;
                                *target = 
                                    get_system::<SquareMap, _>(&c.observer)
                                    .find_closest(
                                        c.storage(), 
                                        bullet.transform.position(), 
                                        HOMING_MISSLE_SEE_RANGE, 
                                        |x| *get_component::<Team, _>(x) != bullet_team
                                    )
                                ;

                                if let Some(target) = target {
                                    tracked_ships.entry(*target)
                                    .or_insert(HashSet::new())
                                    .insert(bullet_id);
                                }
                            },
                        }
                                
                        if align_timer.my_is_zero() {
                            bullet.transform.move_in_direction(4.5f32 * dt.as_secs_f32());
                        } else {
                            bullet.transform.move_in_direction(1.8f32 * dt.as_secs_f32());
                        }
        
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }

                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, CollisionModelIndex::BulletTester, &bullet.transform.full_transform())
                                {
                                    get_component_mut::<HpInfo, _>(obj).damage(1);
                                    bullet.lifetime = <Duration as DurationExt>::my_zero();
                                } 
                            }
                        )
                    },
                }
            } 
        );

        self.mem.retain(
            |bullet_id, bullet| {
                // Determine what is illegal
                // for the bullet to live
                match bullet.kind {
                    BulletData::TestBullet | BulletData::LaserBall | 
                    BulletData::SpinningLaser | BulletData::LaserBeam => {
                        !bullet.lifetime.my_is_zero()
                    },
                    BulletData::HomingMissle { target, .. } => {
                        if bullet.lifetime.my_is_zero() {
                            target.map(
                                |x| 
                                tracked_ships.get_mut(&x)
                                .unwrap().remove(&bullet_id)
                            );
                            false
                        } else { true }
                    },
                }
            }
        );
    }
}

impl<S : Storage> DeletionObserver<S> for BulletSystem {
    fn on_delete(&mut self, _storage : &mut S, idx : usize) {
        if let 
            Some(bullets) =
            self.tracked_ships
            .remove(&idx)
        {
            bullets.into_iter()
            .for_each(|bullet_id| 
                match &mut self.mem.get_mut(bullet_id).unwrap().kind {
                    BulletData::HomingMissle { target, .. } => *target = None,
                    // TODO a warning
                    _ => (),
                }
            )
        }
    }
}
