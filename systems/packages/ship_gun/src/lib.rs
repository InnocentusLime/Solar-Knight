use std::time::Duration;
use std::collections::{ HashMap, HashSet };

use slab::Slab;
use tinyvec::ArrayVec;

use teams::Team;
use hp_system::HpInfo;
use square_map::{ SquareMap, SquareMapNode };
use collision_check::{ CollisionSystem, CollisionInfo };
use systems_core::{ ComponentAccess, get_component, get_component_mut, Observation, MutationObserver, DeletionObserver, Storage, SystemAccess, get_system };
use ship_transform::Transform;
use sys_api::graphics_init::PLAYER_BULLET_LIMIT;
use cgmath_ext::{ rotate_vector_ox, rotate_vector_oy, rotate_vector_angle };
use std_ext::duration_ext::*;
use cgmath_ext::{ matrix3_from_translation };

use cgmath::{ Point2, Vector2, Matrix3, Matrix4, EuclideanSpace, InnerSpace, vec2 };

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
    pub pos : Point2<f32>,
    pub direction : Vector2<f32>,
    pub kind : BulletData,
    pub lifetime : Duration,
    pub team : Team,
    pub parent : usize,
}

impl Bullet {
    const HOMING_MISSLE_ALIGN_TIME : Duration = Duration::from_millis(500);
    
    pub fn size(&self) -> (f32, f32) {
        use sys_api::graphics_init::SCREEN_WIDTH;
        match self.kind {
            BulletData::TestBullet => (0.06f32, 0.09f32),
            BulletData::LaserBall => (0.03f32, 0.03f32),
            BulletData::SpinningLaser { .. } | BulletData::LaserBeam { .. } => (0.03f32, SCREEN_WIDTH / 1.5f32),
            BulletData::HomingMissle { .. } => (0.06f32, 0.09f32),
        }
    }

    #[inline]
    pub fn model_mat(&self) -> Matrix4<f32> {
        use cgmath::One;
        use sys_api::graphics_init::SCREEN_WIDTH;
        
        let size = self.size();

        Matrix4::from_translation(self.pos.to_vec().extend(0.0f32)) * 
        Matrix4::new(
            self.direction.y, -self.direction.x, 0.0f32, 0.0f32,
            self.direction.x, self.direction.y, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ) *
        (
            match self.kind {
                BulletData::SpinningLaser { .. } | BulletData::LaserBeam { .. } => {
                    Matrix4::from_translation(vec2(0.0f32, SCREEN_WIDTH / 1.5f32).extend(0.0f32))
                },
                _ => Matrix4::one(),
            }
        ) *
        Matrix4::from_nonuniform_scale(size.0, size.1, 1.0f32)
    }

    #[inline]
    pub fn transform(&self) -> Matrix3<f32> {
        matrix3_from_translation(self.pos.to_vec()) *
        Matrix3::new(
            self.direction.y, -self.direction.x, 0.0f32,
            self.direction.x, self.direction.y, 0.0f32,
            0.0f32, 0.0f32, 1.0f32,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gun {
    offset : Vector2<f32>,
    bullet_kind : BulletKind,
    recoil : Duration,
    timer : Duration,
    direction : Vector2<f32>,
}

impl Gun {
    pub fn new(
        offset : Vector2<f32>,
        bullet_kind : BulletKind,
        recoil : Duration,
        direction : Vector2<f32>,
    ) -> Self {
        // FIXME direction check

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
            vec2(0.0f32, 0.0f32),
            BulletKind::TestBullet,
            <Duration as DurationExt>::my_zero(),
            vec2(0.0f32, 1.0f32)
        )
    }
}

pub const ENGINE_LIMIT : usize = 5;
#[derive(Clone, Copy, Debug)]
pub struct Guns {
    pub guns : ArrayVec<[Gun; ENGINE_LIMIT]>,
}
    
#[inline]
fn rotate_towards(
    pos : Point2<f32>,
    direc : Vector2<f32>,
    target : Point2<f32>,
    angular_speed : f32,
    dt : Duration,
) -> Vector2<f32> {
    let dir_vec = (target - pos).normalize();
    let ang = direc.angle(dir_vec);

    if ang.0.abs() > angular_speed * dt.as_secs_f32() {
        let (c, s) =
            if ang.0 > 0.0f32 {
                ((angular_speed * dt.as_secs_f32()).cos(), (angular_speed * dt.as_secs_f32()).sin())
            } else {
                ((angular_speed * dt.as_secs_f32()).cos(), -(angular_speed * dt.as_secs_f32()).sin())
            } 
        ;
        rotate_vector_ox(direc, vec2(c, s))
    } else { dir_vec }
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
            (transform.pos, transform.direction())
        }; 
        let shooter_team = *get_component::<Team, _>(obj);

        let gun = get_component_mut::<Guns, _>(obj).guns.get_mut(gun).unwrap();

        let off = rotate_vector_oy(shooter_dir, gun.offset);
        let bullet_dir = rotate_vector_oy(shooter_dir, gun.direction);

        if !gun.can_shoot() { return; }; gun.shoot();

        match gun.kind() {
            BulletKind::TestBullet => {
                self.spawn(        
                    Bullet {
                        pos : shooter_pos + off,
                        team : shooter_team,
                        direction : bullet_dir,
                        kind : BulletData::TestBullet,
                        lifetime : Duration::from_secs(3),
                        parent,
                    }
                )
            },
            BulletKind::LaserBall => {
                self.spawn(        
                    Bullet {
                        pos : shooter_pos + off,
                        team : shooter_team,
                        direction : bullet_dir,
                        kind : BulletData::LaserBall,
                        lifetime : Duration::from_secs(3),
                        parent,
                    }
                )
            },
            BulletKind::SpinningLaser => {
                self.spawn(        
                    Bullet {
                        pos : shooter_pos + off,
                        team : shooter_team,
                        direction : bullet_dir,
                        kind : BulletData::SpinningLaser,
                        lifetime : Duration::from_secs(1),
                        parent,
                    }
                )
            },
            BulletKind::LaserBeam => {
                self.spawn(        
                    Bullet {
                        pos : shooter_pos + off,
                        team : shooter_team,
                        direction : bullet_dir,
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
                            pos : shooter_pos + off,
                            team : shooter_team,
                            direction : direc,
                            kind : BulletData::HomingMissle { 
                                target : None,
                                align_timer : Bullet::HOMING_MISSLE_ALIGN_TIME,
                            },
                            lifetime : Duration::from_secs(3),
                            parent,
                        }
                    )
                ;
                spawn_bullet(rotate_vector_angle(shooter_dir, std::f32::consts::PI * 0.9f32));
                spawn_bullet(rotate_vector_angle(shooter_dir, -std::f32::consts::PI * 0.9f32));
                spawn_bullet(rotate_vector_angle(shooter_dir, std::f32::consts::PI * 0.7f32));
                spawn_bullet(rotate_vector_angle(shooter_dir, -std::f32::consts::PI * 0.7f32));
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
        use collision::*;
        use std_ext::*;
        use collision_check::consts::{ BulletTester, LaserBall, LaserBeam };

        let tracked_ships = &mut self.tracked_ships;
        self.mem.iter_mut()
        .for_each(
            |(bullet_id, bullet)| {
                debug_assert!((bullet.direction.magnitude() - 1.0f32) < VECTOR_NORMALIZATION_RANGE);

                bullet.lifetime = bullet.lifetime.my_saturating_sub(dt);

                // Update bullet data and damage on collision
                match &mut bullet.kind {
                    // TestBullet's move towards with speed 
                    // equal to 4.0.
                    BulletData::TestBullet => {
                        bullet.pos += (4.0f32 * dt.as_secs_f32()) * bullet.direction;
        
                        let my_body = BulletTester.apply_transform(&bullet.transform());
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }
            
                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, &my_body)
                                {
                                    get_component_mut::<HpInfo, _>(obj).damage(1);
                                    bullet.lifetime = <Duration as DurationExt>::my_zero();
                                } 
                            }
                        )
                    },        
                    BulletData::LaserBall => {
                        bullet.pos += (0.7f32 * dt.as_secs_f32()) * bullet.direction;
        
                        let my_body = LaserBall.apply_transform(&bullet.transform());
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }

                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, &my_body)
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
                            .map(|obj| get_component::<Transform, _>(obj).pos)
                            .unwrap()
                        ;
                        bullet.pos = parent_pos;
                        let (sin, cos) = (std::f32::consts::TAU * dt.as_secs_f32()).sin_cos();
                        bullet.direction = rotate_vector_ox(bullet.direction, vec2(cos, sin));
                        
                        let my_body = LaserBeam.apply_transform(&bullet.transform());
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }
            
                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, &my_body)
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
                                (transform.pos, transform.direction()) 
                            }).unwrap()
                        ;
                        bullet.pos = parent_pos;
                        bullet.direction = parent_direction;
                        
                        let my_body = LaserBeam.apply_transform(&bullet.transform());
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }
            
                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, &my_body)
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
                                    ).pos
                                ;
                                if align_timer.my_is_zero() {
                                    bullet.direction = (target - bullet.pos).normalize();
                                } else {
                                    *align_timer = align_timer.saturating_sub(dt);
                                    bullet.direction = rotate_towards(bullet.pos, bullet.direction, target, std::f32::consts::TAU * 0.7f32, dt);
                                }
                            },
                            None => { 
                                let bullet_team = bullet.team;
                                *target = 
                                    get_system::<SquareMap, _>(&c.observer)
                                    .find_closest(
                                        c.storage(), 
                                        bullet.pos, 
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
                            bullet.pos += (4.5f32 * dt.as_secs_f32()) * bullet.direction;
                        } else {
                            bullet.pos += (1.8f32 * dt.as_secs_f32()) * bullet.direction;
                        }
        
                        let my_body = BulletTester.apply_transform(&bullet.transform());
                        //let my_aabb = my_body.aabb();

                        c.mutate_each(
                            |obj, _| {
                                if bullet.lifetime.my_is_zero() { return }

                                if 
                                    *get_component::<Team, _>(obj) != bullet.team &&
                                    get_component::<HpInfo, _>(obj).is_alive() &&
                                    //target_aabb.collision_test(my_aabb) && 
                                    collision.check(obj, &my_body)
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
