use cgmath::{ Vector2, vec2 };

use ship_parts::{ declare_engine, declare_gun, declare_ships, exponential_decrease_curve };

declare_engine!(
    snappy_engine TesterEngine { 
        speed_mul : 0.06f32, 
        max_lvl : 1, 
        direction : (0.0f32, 1.0f32),
    }
);

declare_gun!(
    inf_gun TestGun {
        offset : cgmath::vec2(0.0f32, 0.0f32),
        bullet_kind : tester_bullet,
        recoil : std::time::Duration::from_millis(133),
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

pub fn enemy_tester_ai(
    _layout : &mut EnemyTester, 
    core : &mut ship_parts::core::Core, 
    others : &std_ext::ExtractResultMut<Ship<ShipLayout>>, 
    _bullet_system : &mut ship_parts::gun::BulletSystem,
    _dt : std::time::Duration,
) {
    use cgmath::InnerSpace;

    let player : ShipBorrow<PlayerShip> = others[0].downcast().unwrap();
    core.direction = (player.core.pos - core.pos).normalize();
}

declare_ships!(
    ship PlayerShip (player_ship) {
        [engines]
        main_engine : PlayerEngine[start=0],
        dasher : PlayerDash[start=0],
        [guns]
        gun : TestGun,
        [ai = no_ai::<PlayerShip>; data = ()]
        [sprite_size = (0.1f32, 0.1f32)]
        [spawn_hp = 3; collision = Player]
    }

    ship EnemyTester (enemy_tester) {
        [engines]
        main_engine : TesterEnemyEngine[start=1],
        [guns]
        [ai = enemy_tester_ai; data = ()]
        [sprite_size = (0.1f32, 0.1f32)]
        [spawn_hp = 3; collision = EnemyTester]
    }
);

// Shortcut-controls for player's layout
impl<'a> ShipBorrowMut<'a, PlayerShip> {
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
}

impl<'a> ShipBorrow<'a, PlayerShip> {
    #[inline]
    pub fn is_dashing(&self) -> bool {
        self.layout.dasher.is_changing()
    }
    
    #[inline]
    pub fn dash_trace_param(&self) -> Option<f32> {
        if self.is_dashing() { Some(self.layout.dasher.get_speed()) }
        else { None }
    }
}
