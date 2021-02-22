pub mod engine;
pub mod gun;
pub mod core;
pub mod collision_models;
pub mod constants;

#[macro_export]
macro_rules! declare_ships {
    (
        $(
        ship $name:ident ($con:ident) {
            [engines]
            $( $engine_name:ident : $engine_type:ty[start = $start_expr : expr], )*
            [guns]
            $( $gun_name:ident : $gun_type:ty, )*
            [ai = $ai_proc:expr; data = $ai_data:ty]
            [sprite_size = ($spr_x:expr, $spr_y:expr)] 
            [spawn_hp = $spawn_hp:expr; collision = $collision:ident]
        }
        )+
    ) => {
        fn no_ai<T>(
            _layout : &mut T, 
            _core : &$crate::core::Core, 
            _others : &std_ext::ExtractResultMut<Ship<ShipLayout>>, 
            _bullet_system : &mut $crate::gun::BulletSystem,
            _dt : std::time::Duration,
        ) {}
        
        $(
            pub struct $name {
                $( pub $engine_name : $engine_type, )*
                $( pub $gun_name : $gun_type, )*
                pub ai_data : $ai_data,
            }

            impl $name {
                const SPRITE_SIZE : (f32, f32) = ($spr_x, $spr_y);
                const AI_PROC : 
                    fn(
                        &mut $name, 
                        &$crate::core::Core, 
                        &std_ext::ExtractResultMut<Ship<ShipLayout>>, 
                        &mut $crate::gun::BulletSystem, 
                        std::time::Duration
                    ) 
                = $ai_proc;

                #[inline]
                pub fn new() -> Self {
                    $name {
                        $( $engine_name : <$engine_type>::new($start_expr), )*
                        $( $gun_name : <$gun_type>::new(), )*
                        ai_data : <$ai_data>::default(),
                    }
                }

                #[inline]
                pub fn update(&mut self, core : &mut $crate::core::Core, dt : std::time::Duration) {
                    $(
                        self.$engine_name.update(core, dt);
                    )*
                    $(
                        self.$gun_name.update(dt);
                    )*
                }

                #[inline(always)]
                pub fn think(
                    &mut self, 
                    core : &$crate::core::Core, 
                    others : &std_ext::ExtractResultMut<Ship<ShipLayout>>, 
                    bullet_system : &mut $crate::gun::BulletSystem,
                    dt : std::time::Duration
                ) { 
                    (Self::AI_PROC)(self, core, others, bullet_system, dt) 
                }
            }
        )+

        pub enum ShipLayout {
            $( $name($name), )+
        }

        impl ShipLayout {
            #[inline]
            pub fn update(&mut self, core : &mut $crate::core::Core, dt : std::time::Duration) {
                match self {
                $(
                    ShipLayout::$name(l) => l.update(core, dt),
                )+
                }
            }
            
            #[inline]
            pub fn think(
                &mut self, 
                core : &$crate::core::Core, 
                others : &std_ext::ExtractResultMut<Ship<ShipLayout>>, 
                bullet_system : &mut $crate::gun::BulletSystem,
                dt : std::time::Duration,
            ) {
                match self {
                $(
                    ShipLayout::$name(l) => l.think(core, others, bullet_system, dt),
                )+
                }
            }
        }

        pub struct Ship<S> {
            pub core : $crate::core::Core,
            pub layout : S,
        }

        impl Ship<ShipLayout> {
            $(
            #[inline]
            pub fn $con(team : $crate::core::Team, pos : cgmath::Point2<f32>, dir : cgmath::Vector2<f32>) -> Ship<ShipLayout> {
                Ship {
                    layout : ShipLayout::$name($name::new()),
                    core : $crate::core::Core::new($spawn_hp, $crate::collision_models::model_indices::CollisionModelIndex::$collision, team, pos, dir),
                }
            }
            )+

            #[inline]
            pub fn update(&mut self, dt : std::time::Duration) {
                self.layout.update(&mut self.core, dt)
            }
            
            #[inline]
            pub fn think(
                &mut self, 
                others : &std_ext::ExtractResultMut<Ship<ShipLayout>>, 
                bullet_system : &mut $crate::gun::BulletSystem,
                dt : std::time::Duration
            ) {
                self.layout.think(&self.core, others, bullet_system, dt)
            }

            pub fn sprite_size(&self) -> (f32, f32) {
                match self.layout {
                $(
                    ShipLayout::$name(_) => $name::SPRITE_SIZE,
                )+
                }
            }

            #[inline]
            pub fn model_mat(&self) -> cgmath::Matrix4<f32> {       
                use cgmath::{ Matrix4, EuclideanSpace };

                let size = self.sprite_size();

                Matrix4::from_translation(self.core.pos.to_vec().extend(0.0f32)) * 
                Matrix4::new(
                    self.core.direction.y, -self.core.direction.x, 0.0f32, 0.0f32,
                    self.core.direction.x, self.core.direction.y, 0.0f32, 0.0f32,
                    0.0f32, 0.0f32, 1.0f32, 0.0f32,
                    0.0f32, 0.0f32, 0.0f32, 1.0f32,
                ) * 
                Matrix4::from_nonuniform_scale(size.0, size.1, 1.0f32)
            }
        }

        pub struct Battlefield {
            ships : std_ext::collections::MemoryChunk<Ship<ShipLayout>>,
        }

        impl Battlefield {
            pub fn new() -> Self {
                use sys_api::graphics_init::ENEMY_LIMIT;

                Battlefield {
                    ships : std_ext::collections::MemoryChunk::with_capacity(ENEMY_LIMIT),
                }
            }

            pub fn spawn(&mut self, ship : Ship<ShipLayout>) {
                self.ships.push(ship);
            }
            
            #[inline]
            pub fn player_mut(&mut self) -> Option<&mut Ship<ShipLayout>> {
                self.ships.as_mut_slice().first_mut()
            }
            
            #[inline]
            pub fn player_downcasted_mut<S>(&mut self) -> Option<ShipBorrowMut<S>> 
            where
                Ship<ShipLayout> : ShipDowncast<S>,
            {
                self.ships.as_mut_slice()
                .first_mut()
                .and_then(|x| x.downcast_mut())
            }

            #[inline]
            pub fn player(&self) -> Option<&Ship<ShipLayout>> {
                self.ships.as_slice().first()
            }
            
            #[inline]
            pub fn player_downcasted<S>(&self) -> Option<ShipBorrow<S>> 
            where
                Ship<ShipLayout> : ShipDowncast<S>,
            {
                self.ships.as_slice()
                .first()
                .and_then(|x| x.downcast())
            }

            pub fn update(&mut self, dt : std::time::Duration) {
                self.ships
                .iter_mut()
                .for_each(|x| x.update(dt));

                self.ships.retain(|x| x.core.hp() > 0);
            }

            pub fn think(&mut self, bullet_system : &mut $crate::gun::BulletSystem, dt : std::time::Duration) {
                use std_ext::*;

                for i in 0..self.ships.len() {
                    let (extract, elem) = self.ships.as_mut_slice().extract_mut(i);

                    if elem.core.is_alive() {
                        elem.think(&extract, bullet_system, dt);
                    }
                }
            }

            pub fn fill_buffer(&self, buff : &mut glium::VertexBuffer<sys_api::basic_graphics_data::SpriteData>) {
                use sys_api::graphics_init::ENEMY_LIMIT;
                
                let mut ptr = buff.map_write();

                if ptr.len() < ENEMY_LIMIT { panic!("Buffer too small"); }

                for i in 0..ptr.len() { 
                    use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
            
                    ptr.set(i, ZEROED_SPRITE_DATA);
                }

                self.ships.iter()
                .enumerate()
                .for_each(|(i, x)| {
                    let m = x.model_mat();
            
                    let dat =
                        sys_api::basic_graphics_data::SpriteData {
                            mat_col1 : m.x.into(),
                            mat_col2 : m.y.into(),
                            mat_col3 : m.z.into(),
                            mat_col4 : m.w.into(),
                            texture_bottom_left : [0.0f32, 0.0f32],
                            texture_top_right : [1.0f32, 1.0f32],
                        }
                    ;
            
                    ptr.set(i, dat);
                });
            }
        }

        use std::slice::IterMut;
        use std::iter::Map;
        impl<'a> $crate::gun::TargetSystem<'a> for Battlefield {
            type Iter = Map<IterMut<'a, Ship<ShipLayout>>, fn(&mut Ship<ShipLayout>) -> &mut $crate::core::Core>;

            fn entity_iterator(&'a mut self) -> Self::Iter {
                self.ships.iter_mut().map(|x| &mut x.core)
            }
        }

        #[derive(Clone, Copy)]
        pub struct ShipBorrow<'a, S> {
            pub core : &'a $crate::core::Core,
            pub layout : &'a S,
        }
        
        pub struct ShipBorrowMut<'a, S> {
            pub core : &'a mut $crate::core::Core,
            pub layout : &'a mut S,
        }

        impl<'a, S> ShipBorrowMut<'a, S> {
            #[inline]
            pub fn downgrade(&self) -> ShipBorrow<S> {
                ShipBorrow {
                    core : &*self.core,
                    layout : &*self.layout,
                }
            }
        }

        pub trait ShipDowncast<S> {
            fn downcast(&self) -> Option<ShipBorrow<S>>;
            fn downcast_mut(&mut self) -> Option<ShipBorrowMut<S>>;
        }

        $(
        impl ShipDowncast<$name> for Ship<ShipLayout> {
            #[inline]
            fn downcast(&self) -> Option<ShipBorrow<$name>> {
                match &self.layout {
                    ShipLayout::$name(layout) => Some(ShipBorrow { core : &self.core, layout }),
                    _ => None,
                }
            }
            
            #[inline]
            fn downcast_mut(&mut self) -> Option<ShipBorrowMut<$name>> {
                match &mut self.layout {
                    ShipLayout::$name(layout) => Some(ShipBorrowMut { core : &mut self.core, layout }),
                    _ => None,
                }
            }
        }
        )+
    };
}
