#[derive(Clone, Copy, Debug)]
pub struct StorageAccessError;

// TODO objects must be able to remember their IDs
pub trait Storage {
    type Object;

    fn delete(&mut self, idx : usize) -> Option<Self::Object>;
    fn spawn(&mut self, obj : Self::Object) -> usize;
    // TODO rename to "max_id"
    fn capacity(&self) -> usize;
    fn get(&self, idx : usize) -> Option<&Self::Object>;
    fn get_mut(&mut self, idx : usize) -> Option<&mut Self::Object>;

    #[inline]
    fn for_each<F : FnMut(&Self::Object)>(&self, mut f : F) {
        (0..self.capacity())
        .filter_map(|i| self.get(i))
        .for_each(|x| f(x))
    }
}

pub trait ComponentAccess<C> {
    fn access(&self) -> &C;
    fn access_mut(&mut self) -> &mut C;
}

pub fn get_component<C, Obj : ComponentAccess<C>>(obj : &Obj) -> &C { obj.access() }
pub fn get_component_mut<C, Obj : ComponentAccess<C>>(obj : &mut Obj) -> &mut C { obj.access_mut() }

pub trait SystemAccess<Sys> {
    fn access(&self) -> &Sys;
    fn access_mut(&mut self) -> &mut Sys;
}

pub fn get_system<C, Obs : SystemAccess<C>>(obs : &Obs) -> &C { obs.access() }
pub fn get_system_mut<C, Obs : SystemAccess<C>>(obs : &mut Obs) -> &mut C { obs.access_mut() }

pub trait DeletionObserver<S : Storage> {
    fn on_delete(&mut self, storage : &mut S, idx : usize);
}

pub trait SpawningObserver<S : Storage> : DeletionObserver<S> {
    fn on_spawn(&mut self, storage : &mut S, idx : usize);
}

// NOTE usually there shouldn't be a lot of mutation observers
// well, because there shouldn't be much data constantly synchronized
// BUT, if it *does* happen, we should consider making observers
// for different sets of components to amortize the overhead.
pub trait MutationObserver<S : Storage> : DeletionObserver<S> {
    fn on_mutation(&mut self, storage : &mut S, idx : usize);
}

pub struct Observation<'a, Observer, S : Storage> {
    me : &'a mut S,
    pub observer : Observer,
}

impl<'a, Observer, S : Storage> Observation<'a, Observer, S> {
    pub fn new(me : &'a mut S, observer : Observer) -> Self {
        Observation {
            me,
            observer,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize { self.me.capacity() }
    
    #[inline]
    pub fn get(&self, id : usize) -> Option<&S::Object> { self.me.get(id) }

    #[inline]
    pub fn storage(&self) -> &S { self.me }
}

impl<'a, S : Storage, Observer : DeletionObserver<S>> Observation<'a, Observer, S> {
    #[inline]
    pub fn delete(&mut self, idx : usize) -> Option<S::Object> {
        self.observer.on_delete(&mut self.me, idx);

        self.me.delete(idx)
    }

    #[inline]
    pub fn retain<F : Fn(&S::Object) -> bool>(&mut self, f : F) {
        for idx in 0..self.me.capacity() {
            if self.me.get(idx).map(|x| !f(x)).unwrap_or(false) {
                self.delete(idx);
            }
        }
    }
}

impl<'a, S : Storage, Observer : SpawningObserver<S>> Observation<'a, Observer, S> {
    pub fn spawn(&mut self, ship : S::Object) -> usize {
        let idx = self.me.spawn(ship);
        
        self.observer.on_spawn(&mut self.me, idx);
        idx
    }
}
   
impl<'a, S : Storage, Observer : MutationObserver<S>> Observation<'a, Observer, S> {
    #[inline]
    pub fn mutate<T, F : FnMut(&mut S::Object, &mut Observer) -> T>(&mut self, idx : usize, mut f : F) -> Option<T> {
        let observer = &mut self.observer;
        let me = &mut self.me;
        me.get_mut(idx)
        .map(|x| f(x, observer))
        .map(|x| { observer.on_mutation(me, idx); x })
    }

    // TODO we might want to have a reaction if the ship isn't present
    pub fn mutate_range<F : FnMut(&mut S::Object, &mut Observer), I : Iterator<Item = usize>>(&mut self, mut f : F, it : I) {
        for idx in it {
            self.mutate(idx, &mut f);
        }
    }
    
    #[inline]
    pub fn mutate_each<F : FnMut(&mut S::Object, &mut Observer)>(&mut self, f : F) {
        self.mutate_range(f, 0..self.me.capacity())
    }
}

#[macro_export]
macro_rules! declare_object {
    (
    $( #[derive( $($trait:ident),+ ) ] )?
    pub object $obj:ident { 
        $($comp_name:ident : $comp_ty:ty),* $(,)?
    }
    ) => {
        $(#[derive($($trait),+)])?
        pub struct $obj {
            $($comp_name : $comp_ty),*
        }

        $(
        impl $crate::ComponentAccess<$comp_ty> for $obj {
            #[inline(always)]
            fn access(&self) -> &$comp_ty { &self.$comp_name }
            #[inline(always)]
            fn access_mut(&mut self) -> &mut $comp_ty { &mut self.$comp_name }
        }
        )*
    }
}

#[macro_export]
macro_rules! declare_observers {
    (
    type Host = $storage:ty;

    pub observer SpawningObserverPack {
        $($spawn_name:ident : $spawn_obs:ty),* $(,)?
    }
    
    pub observer DeletionObserverPack {
        $($delete_name:ident : $delete_obs:ty),* $(,)?
    }
    
    pub observer MutationObserverPack {
        $($mutate_name:ident : $mutate_obs:ty),* $(,)?
    }
    ) => {
        pub struct DeletionObserverPack<'a> {
            _phantom : std::marker::PhantomData<fn(&'a mut $storage)>
            $(, pub $delete_name : &'a mut $delete_obs)*
        }

        impl<'a> $crate::DeletionObserver<$storage> for DeletionObserverPack<'a> 
        {
            fn on_delete(&mut self, storage : &mut $storage, idx : usize) {
                $(self.$delete_name.on_delete(storage, idx));*
            }
        }

        $(
        impl<'a> $crate::SystemAccess<$delete_obs> for DeletionObserverPack<'a> {
            fn access(&self) -> &$delete_obs { &self.$delete_name }
            fn access_mut(&mut self) -> &mut $delete_obs { &mut self.$delete_name }
        }
        )*
        
        pub struct SpawningObserverPack<'a> {
            _phantom : std::marker::PhantomData<fn(&'a mut $storage)>
            $(, pub $spawn_name : &'a mut $spawn_obs)*
        }
        
        impl<'a> $crate::DeletionObserver<$storage> for SpawningObserverPack<'a> 
        {
            fn on_delete(&mut self, storage : &mut $storage, idx : usize) {
                $(self.$spawn_name.on_delete(storage, idx));*
            }
        }

        impl<'a> $crate::SpawningObserver<$storage> for SpawningObserverPack<'a> 
        {
            fn on_spawn(&mut self, storage : &mut $storage, idx : usize) {
                $(self.$spawn_name.on_spawn(storage, idx));*
            }
        }
        
        $(
        impl<'a> $crate::SystemAccess<$spawn_obs> for SpawningObserverPack<'a> {
            fn access(&self) -> &$spawn_obs { &self.$spawn_name }
            fn access_mut(&mut self) -> &mut $spawn_obs { &mut self.$spawn_name }
        }
        )*

        pub struct MutationObserverPack<'a> {
            _phantom : std::marker::PhantomData<fn(&'a mut $storage)>
            $(, pub $mutate_name : &'a mut $mutate_obs)*
        }
        
        impl<'a> $crate::DeletionObserver<$storage> for MutationObserverPack<'a> 
        {
            fn on_delete(&mut self, storage : &mut $storage, idx : usize) {
                $(self.$mutate_name.on_delete(storage, idx));*;
            }
        }

        impl<'a> $crate::MutationObserver<$storage> for MutationObserverPack<'a> 
        {
            fn on_mutation(&mut self, storage : &mut $storage, idx : usize) {
                $(self.$mutate_name.on_mutation(storage, idx));*
            }
        }
         
        $(
        impl<'a> $crate::SystemAccess<$mutate_obs> for MutationObserverPack<'a> {
            #[inline(always)]
            fn access(&self) -> &$mutate_obs { &self.$mutate_name }
            #[inline(always)]
            fn access_mut(&mut self) -> &mut $mutate_obs { &mut self.$mutate_name }
        }
        )*

        impl $storage {
            pub fn unlock_deletion<'a>(
                &'a mut self,
                $($delete_name : &'a mut $delete_obs),*
            ) -> $crate::Observation<'a, DeletionObserverPack, $storage> {
                $crate::Observation::new(
                    self,
                    DeletionObserverPack {
                        _phantom : std::marker::PhantomData
                        $(, $delete_name)*
                    }
                )
            }
            
            pub fn unlock_spawning<'a>(
                &'a mut self,
                $($spawn_name : &'a mut $spawn_obs),*
            ) -> $crate::Observation<'a, SpawningObserverPack, $storage> {
                $crate::Observation::new(
                    self,
                    SpawningObserverPack {
                        _phantom : std::marker::PhantomData
                        $(, $spawn_name)*
                    }
                )
            }
            
            pub fn unlock_mutations<'a>(
                &'a mut self,
                $($mutate_name : &'a mut $mutate_obs),*
            ) -> $crate::Observation<'a, MutationObserverPack, $storage> {
                $crate::Observation::new(
                    self,
                    MutationObserverPack {
                        _phantom : std::marker::PhantomData
                        $(, $mutate_name)*
                    }
                )
            }
        }
    };
}
