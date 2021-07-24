use crate::storage::{ Ship, Storage, MutableStorage };

// TODO Generic `GET` function. All systems should modify only
// their own components when they reply to mutations. Observers
// shouldn't be able to read each other's components when reacting,
// which leads us to the following protocol:
// 1. Mutate only your stuff
// 2. You can read `Core`

pub trait DeletionObserver {
    fn on_delete(&mut self, storage : &mut MutableStorage, idx : usize);
}

pub trait SpawningObserver : DeletionObserver {
    fn on_spawn(&mut self, storage : &mut MutableStorage, idx : usize);
}

pub trait MutationObserver : DeletionObserver {
    fn on_mutation(&mut self, storage : &mut MutableStorage, idx : usize);
}

pub struct Observation<'a, Observer> {
    me : &'a mut Storage,
    observer : Observer,
}

impl<'a, Observer> Observation<'a, Observer> {
    pub(crate) fn new(me : &'a mut Storage, observer : Observer) -> Self {
        Observation {
            me,
            observer,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize { self.me.capacity() }
    
    #[inline]
    pub fn get(&self, id : usize) -> Option<&Ship> { self.me.get(id) }

    #[inline]
    pub fn storage(&self) -> &Storage { self.me }

    #[inline]
    pub fn observer(&self) -> &Observer { &self.observer }
}

impl<'a, Observer : DeletionObserver> Observation<'a, Observer> {
    #[inline]
    pub fn delete(&mut self, idx : usize) -> Option<Ship> {
        let mut mutator = self.me.mutate();
        self.observer.on_delete(&mut mutator, idx);

        self.me.delete(idx)
    }

    #[inline]
    pub fn filter<F : Fn(&Ship) -> bool>(&mut self, f : F) {
        for idx in (0..self.me.capacity()) {
            if self.me.get(idx).map(|x| f(x)).unwrap_or(false) {
                self.delete(idx);
            }
        }
    }
}

impl<'a, Observer : SpawningObserver> Observation<'a, Observer> {
    pub fn spawn(&mut self, ship : Ship) {
        let idx = self.me.spawn(ship);
        
        let mut mutator = self.me.mutate();
        self.observer.on_spawn(&mut mutator, idx);
    }
    
    // TODO shouldn't exist here.
    pub fn spawn_template(&mut self, template_id : usize) {
        let idx = self.me.spawn_template(template_id);
        
        let mut mutator = self.me.mutate();
        self.observer.on_spawn(&mut mutator, idx);
    }
}
   
impl<'a, Observer : MutationObserver> Observation<'a, Observer> {
    #[inline]
    pub fn mutate<T, F : FnMut(&mut Ship) -> T>(&mut self, idx : usize, mut f : F) -> Option<T> {
        let observer = &mut self.observer;
        let mut mutator = self.me.mutate();
    
        mutator.get_mut(idx)
        .map(|x| f(x))
        .map(|x| { observer.on_mutation(&mut mutator, idx); x })
    }

    // TODO we might want to have a reaction if the ship isn't present
    pub fn mutate_range<F : FnMut(&mut Ship), I : Iterator<Item = usize>>(&mut self, mut f : F, it : I) {
        for idx in it {
            self.mutate(idx, &mut f);
        }
    }
    
    #[inline]
    pub fn mutate_each<F : FnMut(&mut Ship)>(&mut self, mut f : F) {
        for idx in (0..self.me.capacity()) {
            self.mutate(idx, &mut f);
        }
    }
}

#[macro_export(local_inner_macros)]
macro_rules! declare_observers {
    (
    spawning_observers : {
        $($spawn_name:ident : $spawn_obs:ty),* $(,)?
    },
    deletion_observers : {
        $($delete_name:ident : $delete_obs:ty),* $(,)?
    },
    mutation_observers : {
        $($mutate_name:ident : $mutate_obs:ty),* $(,)?
    },
    ) => {
        pub struct DeletionObserverPack<'a> {
            _phantom : std::marker::PhantomData<&'a mut ()>
            $(, pub $delete_name : &'a mut $delete_obs)*
        }

        impl<'a> DeletionObserver for DeletionObserverPack<'a> {
            fn on_delete(&mut self, storage : &mut MutableStorage, idx : usize) {
                $(self.$delete_name.on_delete(storage, idx));*
            }
        }
        
        pub struct SpawningObserverPack<'a> {
            _phantom : std::marker::PhantomData<&'a mut ()>
            $(, pub $spawn_name : &'a mut $spawn_obs)*
        }
        
        impl<'a> DeletionObserver for SpawningObserverPack<'a> {
            fn on_delete(&mut self, storage : &mut MutableStorage, idx : usize) {
                $(self.$spawn_name.on_delete(storage, idx));*
            }
        }

        impl<'a> SpawningObserver for SpawningObserverPack<'a> {
            fn on_spawn(&mut self, storage : &mut MutableStorage, idx : usize) {
                $(self.$spawn_name.on_spawn(storage, idx));*
            }
        }

        pub struct MutationObserverPack<'a> {
            _phantom : std::marker::PhantomData<&'a mut ()>
            $(, pub $mutate_name : &'a mut $mutate_obs)*
        }
        
        impl<'a> DeletionObserver for MutationObserverPack<'a> {
            fn on_delete(&mut self, storage : &mut MutableStorage, idx : usize) {
                $(self.$mutate_name.on_delete(storage, idx));*;
            }
        }

        impl<'a> MutationObserver for MutationObserverPack<'a> {
            fn on_mutation(&mut self, storage : &mut MutableStorage, idx : usize) {
                $(self.$mutate_name.on_mutation(storage, idx));*
            }
        }

        impl Storage {
            pub fn unlock_deletion<'a>(
                &'a mut self,
                $($delete_name : &'a mut $delete_obs),*
            ) -> Observation<'a, DeletionObserverPack> {
                Observation::new(
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
            ) -> Observation<'a, SpawningObserverPack> {
                Observation::new(
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
            ) -> Observation<'a, MutationObserverPack> {
                Observation::new(
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
