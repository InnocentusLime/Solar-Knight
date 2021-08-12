use slab::Slab;

pub struct Storage<Obj> {
    mem : Slab<Obj>,
}

impl<Obj> Storage<Obj> {
    pub fn new() -> Self {
        Storage {
            mem : Slab::new(),
            /*
            template_table : vec![
                TemplateTableEntry::player_ship(),
                TemplateTableEntry::turret_ship(),
                TemplateTableEntry::heavy_body(),
                TemplateTableEntry::fly_ship(),
            ],
            */
        }
    }
}

impl<Obj> systems::systems_core::Storage for Storage<Obj> {
    type Object = Obj;

    #[inline]
    fn spawn(&mut self, ship : Self::Object) -> usize {
        self.mem.insert(ship)
    }
    
    #[inline]
    fn get(&self, id : usize) -> Option<&Self::Object> { self.mem.get(id) }
    
    #[inline]
    fn get_mut(&mut self, id : usize) -> Option<&mut Self::Object> { self.mem.get_mut(id) }

    #[inline]
    fn delete(&mut self, id : usize) -> Option<Self::Object> {
        if self.mem.contains(id) { 
            Some(self.mem.remove(id)) 
        } else { None }
    }

    #[inline]
    fn capacity(&self) -> usize { self.mem.capacity() }
}
