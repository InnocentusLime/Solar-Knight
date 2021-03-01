use std::any::{ Any, TypeId };
use std::time::Duration;

use crate::core::Core;
use crate::gun::{ BulletSystem, TargetSystem };

use std_ext::{ ExtractResultMut, SliceExt };
use std_ext::collections::MemoryChunk;
use sys_api::basic_graphics_data::SpriteData;

use glium::VertexBuffer;
use cgmath::{ Matrix4, Point2, EuclideanSpace };

pub trait ShipLayout<S> : Copy + Any + 'static {
    fn update(me : &mut Ship<Self>, dt : Duration);
    // TODO each ship probably would like to write its own data
    fn sprite_size() -> (f32, f32);
}

pub unsafe trait SuperShipLayout : Sized + 'static {
    fn upcast<T : ShipLayout<Self>>(x : T) -> Self;
}

const PADDING_SIZE : usize = 0;

#[repr(C)]
pub struct Ship<S : 'static> {
    type_id : TypeId,
    sprite_size : fn() -> (f32, f32),
    update : fn(&mut Ship<S>, Duration),
    pub think : fn(
        me : &mut Ship<S>,
        others : &ExtractResultMut<Ship<S>>,
        bullet_system : &mut BulletSystem,
        dt : Duration
    ),
    pub core : Core,
    _pad : [u8; PADDING_SIZE],
    pub layout : S,
}

impl<S : 'static> Ship<S> {
    fn validate(&self) {
        let offset_layout = 
            ((&self.layout as *const S) as usize) - 
            ((self as *const Ship<S>) as usize)
        ;
        let offset_pad = 
            ((&self._pad as *const [u8; PADDING_SIZE]) as usize) - 
            ((self as *const Ship<S>) as usize)
        ;

        //dbg!(offset_layout); dbg!(offset_pad);

        assert_eq!(offset_layout - offset_pad, PADDING_SIZE, "Bad `PADDING_SIZE` value. (left - actual offset. right - guessed offset)");
    }

    #[inline]
    fn model_mat(&self) -> Matrix4<f32> {
        let size = (self.sprite_size)();

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

impl<S : SuperShipLayout + 'static> Ship<S> {
        
    pub fn new<T : ShipLayout<S>>(
        layout : T, core : Core, 
        think : fn(
            me : &mut Ship<T>,
            others : &ExtractResultMut<Ship<S>>,
            bullet_system : &mut BulletSystem,
            dt : Duration
        )
    ) -> Ship<S> {
        use std::mem;

        let cell =
            unsafe {
                Ship { 
                    type_id : TypeId::of::<T>(),
                    sprite_size : <T as ShipLayout<S>>::sprite_size,
                    update : mem::transmute(<T as ShipLayout<S>>::update as *const ()),
                    think : mem::transmute(think as *const ()),
                    core,
                    _pad : [0; PADDING_SIZE],
                    layout : <S as SuperShipLayout>::upcast::<T>(layout),
                }
            }
        ;
        #[cfg(debug_assertions)] {
            cell.validate();
            let downed = cell.downcast::<T>().expect("Failed to downcast data to T which was an upcasted T");
            downed.validate();
        }
        cell
    }

    pub fn is<T : ShipLayout<S>>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    pub fn downcast<T : ShipLayout<S>>(&self) -> Option<&Ship<T>> {
        use std::mem;

        if self.is::<T>() {
            Some(
                unsafe { mem::transmute(self) }
            )
        } else { None }
    }
    
    pub fn downcast_mut<T : ShipLayout<S>>(&mut self) -> Option<&mut Ship<T>> {
        use std::mem;

        if self.is::<T>() {
            Some(
                unsafe { mem::transmute(self) }
            )
        } else { None }
    }
}

pub struct Battlefield<S : SuperShipLayout + 'static> {
    mem : Vec<Ship<S>>,
}

impl<S : SuperShipLayout + 'static> Battlefield<S> {
    pub fn new() -> Battlefield<S> {
        Battlefield {
            mem : Vec::new(),
        }
    }

    pub fn update(&mut self, dt : Duration) {
        self.mem.iter_mut()
        .for_each(|c| (c.update)(c, dt));

        self.mem.retain(|x| x.core.hp() > 0);
    }
            
    pub fn think(&mut self, bullet_system : &mut BulletSystem, dt : Duration) {
        use std_ext::*;

        for i in 0..self.mem.len() {
            let (extract, elem) = self.mem.as_mut_slice().extract_mut(i);

            if elem.core.is_alive() {
                (elem.think)(elem, &extract, bullet_system, dt);
            }
        }
    }
    
    pub fn spawn(&mut self, ship : Ship<S>) {
        self.mem.push(ship);
    }
            
    pub fn fill_buffer(&self, buff : &mut VertexBuffer<SpriteData>) {
        use sys_api::graphics_init::ENEMY_LIMIT;
                
        let mut ptr = buff.map_write();

        if ptr.len() < ENEMY_LIMIT { panic!("Buffer too small"); }

        for i in 0..ptr.len() { 
            use sys_api::basic_graphics_data::ZEROED_SPRITE_DATA;
            
            ptr.set(i, ZEROED_SPRITE_DATA);
        }

        self.mem.iter()
        .enumerate()
        .for_each(|(i, x)| {
            let m = x.model_mat();
            
            let dat =
                SpriteData {
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

    #[inline]
    pub fn get(&self, id : usize) -> Option<&Ship<S>> { self.mem.get(id) }
    
    #[inline]
    pub fn get_mut(&mut self, id : usize) -> Option<&mut Ship<S>> { self.mem.get_mut(id) }
    
    #[inline]
    pub fn get_downcasted<T : ShipLayout<S>>(&self, id : usize) -> Option<&Ship<T>> { 
        self.mem
        .as_slice()
        .get(id) 
        .and_then(|x| x.downcast::<T>())
    }
    
    #[inline]
    pub fn get_mut_downcasted<T : ShipLayout<S>>(&mut self, id : usize) -> Option<&mut Ship<T>> { 
        self.mem
        .as_mut_slice()
        .get_mut(id) 
        .and_then(|x| x.downcast_mut::<T>())
    }
}
        
use std::slice::IterMut;
use std::iter::Map;
impl<'a, S : 'static + SuperShipLayout> TargetSystem<'a> for Battlefield<S> {
    type Iter = Map<IterMut<'a, Ship<S>>, fn(&mut Ship<S>) -> &mut Core>;

    fn entity_iterator(&'a mut self) -> Self::Iter {
        self.mem.iter_mut().map(|x| &mut x.core)
    }
}
