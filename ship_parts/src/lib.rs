pub mod engine;
pub mod gun;
pub mod core;
pub mod earth;
pub mod collision_models;
pub mod constants;
pub mod storage_traits;
pub mod part_trait;

use crate::part_trait::ShipPart;

pub use crate::core::Team;
pub use crate::earth::Earth;
pub use crate::gun::BulletSystem;
pub use crate::storage_traits::{ Ship, ShipLayout, SuperShipLayout, Battlefield as BattlefieldBase };       

use cgmath::{ Point2, Vector2, vec2, point2 };

fn no_ai<T>(
    _me : &mut Ship<T>,
    _others : &std_ext::ExtractResultMut<ShipObject>, 
    _bullet_system : &mut crate::gun::BulletSystem,
    _earth : &Earth,
    _dt : std::time::Duration,
) {}
        

#[macro_export]
macro_rules! declare_ships {
    (
        $(
        ship $name:ident ($con:ident) {
            $($part_name:ident : $part_type:ty[$($arg:expr),*],)+
            [ai = $ai_proc:expr; data = $ai_data:ty]
            [render = $render_proc:expr] 
            [spawn_hp = $spawn_hp:expr; collision = $collision:ident]
        }
        )+
    ) => {
        $(
            #[derive(Clone, Copy)]
            pub struct $name {
                $( pub $part_name : $part_type, )*
                pub ai_data : $ai_data,
            }

            impl $name {
                #[inline]
                pub fn new() -> Self {
                    $name {
                        $( $part_name : <$part_type>::new($($arg),*), )+
                        ai_data : <$ai_data>::default(),
                    }
                }
            }

            impl ShipLayout<ShipLayoutUnion> for $name {
                #[inline]
                fn update(me : &mut Ship<$name>, dt : std::time::Duration) {
                    $(me.layout.$part_name.update(&mut me.core, dt);)+
                }
            }
            
            pub fn $con(team : Team, pos : cgmath::Point2<f32>, dir : cgmath::Vector2<f32>) -> ShipObject {
                Ship::new(
                    $name::new(), 
                    crate::core::Core::new(
                        $spawn_hp, 
                        crate::collision_models::model_indices::CollisionModelIndex::$collision, 
                        team, pos, dir
                    ),
                    $ai_proc,
                    $render_proc
                )
            }
        )+

        pub union ShipLayoutUnion {
            $( $con : $name, )+
        }

        unsafe impl SuperShipLayout for ShipLayoutUnion {
            fn upcast<T : ShipLayout<Self>>(x : T) -> Self {
                use std::mem;

                unsafe {
                    let mut res = mem::MaybeUninit::zeroed();
                    (&x as *const T).copy_to_nonoverlapping(res.as_mut_ptr() as *mut T, 1);
                    res.assume_init()
                }
            }
        }

        pub type ShipObject = Ship<ShipLayoutUnion>;
        pub type Battlefield = BattlefieldBase<ShipLayoutUnion>;
    };
}

declare_engine!(
    snappy_engine TesterEngine { 
        speed_mul : 0.06f32, 
        max_lvl : 1, 
        direction : (0.0f32, 1.0f32),
    }
);

declare_engine!(
    snappy_engine BruteEngine { 
        speed_mul : 0.08f32, 
        max_lvl : 1, 
        direction : (0.0f32, 1.0f32),
    }
);

declare_gun!(
    inf_gun TestGun {
        offset : cgmath::vec2(0.0f32, 0.0f32),
        bullet_kind : tester_bullet,
        recoil : std::time::Duration::from_millis(350),
        direction : cgmath::vec2(0.0f32, 1.0f32),
    }
);

declare_gun!(
    inf_gun TesterEnemyGun {
        offset : cgmath::vec2(0.0f32, 0.0f32),
        bullet_kind : tester_bullet,
        recoil : std::time::Duration::from_secs(2),
        direction : cgmath::vec2(0.0f32, 1.0f32),
    }
);

declare_gun!(
    inf_gun LaserBallGun {
        offset : cgmath::vec2(0.0f32, 0.0f32),
        bullet_kind : laser_ball,
        recoil : std::time::Duration::from_secs(2),
        direction : cgmath::vec2(0.0f32, 1.0f32),
    }
);

declare_engine!(
    snappy_engine PlayerEngine { 
        speed_mul : 0.5f32, 
        max_lvl : 4, 
        direction : (0.0f32, 1.0f32),
    }
);

declare_engine!(
    snappy_engine TesterEnemyEngine { 
        speed_mul : 0.06f32, 
        max_lvl : 1, 
        direction : (0.0f32, 1.0f32),
    }
);

declare_engine!(
    directed_soft_engine PlayerDash {
        speed_mul : 6.0f32,
        max_lvl : 1,
        one_step_duration : std::time::Duration::from_millis(180),
        change_curve : exponential_decrease_curve!(std::f32::consts::E / 2.1f32),
    }
);

use sys_api::basic_graphics_data::SpriteData;
use sys_api::graphics_init::SpriteDataWriter;
fn test_render<S>(
    me : &Ship<S>,
    buff : &mut SpriteDataWriter,
) {
    let m = me.model_mat((0.1f32, 0.1f32));
    //dbg!(i); dbg!(m);
    
    let color : [f32; 4];
    if me.core.team() == Team::Hive { color = [1.0f32, 0.01f32, 0.01f32, 1.0f32] }
    else { color = [1.0f32; 4] }
            
    let dat =
        SpriteData {
            mat_col1 : m.x.into(),
            mat_col2 : m.y.into(),
            mat_col3 : m.z.into(),
            mat_col4 : m.w.into(),
            texture_bottom_left : [0.0f32, 0.0f32],
            width_height : [1.0f32, 1.0f32],
            color : color,
        }
    ;

    buff.put(dat);
}

pub fn enemy_tester_ai(
    me : &mut Ship<EnemyTester>,
    others : &std_ext::ExtractResultMut<ShipObject>, 
    bullet_system : &mut BulletSystem,
    _earth : &Earth,
    _dt : std::time::Duration,
) {
    use cgmath::InnerSpace;
    let player = &others[0];
    let me_player_vec = (player.core.pos - me.core.pos);
    me.core.direction = me_player_vec.normalize(); 

    if me_player_vec.magnitude() <= 4.0f32 {
        me.layout.gun.shoot(&me.core)
        .map_or((), |x| bullet_system.spawn(x))
    }
}

pub fn enemy_brute_ai(
    me : &mut Ship<EnemyBrute>,
    others : &std_ext::ExtractResultMut<ShipObject>, 
    bullet_system : &mut BulletSystem,
    earth : &Earth,
    _dt : std::time::Duration,
) {
    use cgmath::InnerSpace;
    let player = &others[0];
    let me_player_vec = (player.core.pos - me.core.pos);

    if me_player_vec.magnitude() <= 4.0f32 {
        // FIXME too hacky, rework with `directed gun`
        me.core.direction = me_player_vec.normalize(); 
        me.layout.gun.shoot(&me.core)
        .map_or((), |x| bullet_system.spawn(x));
        me.core.direction = (earth.pos() - me.core.pos).normalize();
    }
}

/*
pub fn enemy_smartie_ai(
    me : &mut Ship<EnemySmartie>,
    others : &std_ext::ExtractResultMut<ShipObject>, 
    bullet_system : &mut BulletSystem,
    earth : &Earth,
    _dt : std::time::Duration,
) {
    const WAIT_TIME : Duration = Duration::from_seconds(4);

    use cgmath::InnerSpace;
    let player = &others[0];
    let me_player_vec = (player.core.pos - me.core.pos);

    if me.core.hp() > 2 {
        if me_player_vec.magnitude() <= 4.0f32 {
            // FIXME too hacky, rework with `directed gun`
            me.core.direction = me_player_vec.normalize(); 
            me.layout.gun.shoot(&me.core)
            .map_or((), |x| bullet_system.spawn(x));
            me.core.direction = (earth.pos() - me.core.pos).normalize();
        }
    } else {

    }
}
*/

declare_ships!(
    ship PlayerShip (player_ship) {
        main_engine : PlayerEngine[0],
        dasher : PlayerDash[0],
        gun : TestGun[],
        [ai = no_ai::<PlayerShip>; data = ()]
        [render = test_render::<PlayerShip>]
        [spawn_hp = 3; collision = Player]
    }

    ship EnemyTester (enemy_tester) {
        main_engine : TesterEnemyEngine[1],
        gun : TesterEnemyGun[],
        [ai = enemy_tester_ai; data = ()]
        [render = test_render::<EnemyTester>]
        [spawn_hp = 3; collision = EnemyTester]
    }

    ship EnemyBrute (enemy_brute) {
        main_engine : BruteEngine[1],
        gun : LaserBallGun[],
        [ai = enemy_brute_ai; data = ()]
        [render = test_render::<EnemyBrute>]
        [spawn_hp = 3; collision = Player]
    }
);

// Shortcut-controls for player's layout
impl Ship<PlayerShip> {
    #[inline]
    pub fn increase_speed(&mut self) { self.layout.main_engine.increase_speed() }
    
    #[inline]
    pub fn decrease_speed(&mut self) { self.layout.main_engine.decrease_speed() }

    #[inline]
    pub fn is_dashing(&self) -> bool {
        self.layout.dasher.is_changing()
    }

    pub fn dash_right(&mut self) -> Option<Vector2<f32>> {
        if !self.is_dashing() {
            // Check `dash_left` for info about how it works
            let direction = cgmath_ext::rotate_vector_ox(self.core.direction, vec2(0.0f32, -1.0f32));
            self.layout.dasher.direction = direction;
            self.layout.dasher.snap(1);
            self.layout.dasher.decrease_speed();
            // return the dash direction
            Some(direction)
        } else { None }
    }
    
    pub fn dash_left(&mut self) -> Option<Vector2<f32>> {
        if !self.is_dashing() {
            // This code will make the engine
            // update all its interior data to be `1`.
            // We'll then call `decrease_speed` to make engine's
            // speed decrease from 1 to 0.
            let direction = cgmath_ext::rotate_vector_ox(self.core.direction, vec2(0.0f32, 1.0f32));
            self.layout.dasher.direction = direction;
            self.layout.dasher.snap(1);
            self.layout.dasher.decrease_speed();
            // return the dash direction
            Some(direction)
        } else { None }
    }
    
    #[inline]
    pub fn dash_trace_param(&self) -> Option<f32> {
        if self.is_dashing() { Some(self.layout.dasher.get_speed()) }
        else { None }
    }
}
