struct Cell<T> {
    val : T,
    next : Option<usize>,
    prev : Option<usize>,
    slot_id : usize,
}

pub struct LinkedLists<T> {
    cells : Vec<Option<Cell<T>>>,
    vacant : Vec<usize>,
    heads : Vec<Option<usize>>,
}

impl<T> LinkedLists<T> {
    pub fn new() -> Self {
        LinkedLists {
            cells : Vec::new(),
            vacant : Vec::new(),
            heads : Vec::new(),
        }
    }

    pub fn with_capacity(cap : usize) -> Self {
        LinkedLists {
            cells : Vec::with_capacity(cap),
            vacant : Vec::with_capacity(cap),
            heads : vec![None; cap],
        }
    }

    pub fn push(&mut self, x : T, slot_id : usize) -> usize {
        let new_cell = Cell { val : x, next : None, prev : None, slot_id };
        let new_id = 
            match self.vacant.pop() {
                Some(vacant) => {
                    self.cells[vacant] = Some(new_cell);
                    vacant
                },
                None => {
                    self.cells.push(Some(new_cell));
                    self.cells.len() - 1
                },
            }
        ;
    
        // Update the head
        match self.heads[slot_id] {
            Some(head) => {
                self.get_cell_mut(new_id).unwrap().next = Some(head);
                self.get_cell_mut(head).unwrap().prev = Some(new_id);
                self.heads[slot_id] = Some(new_id);
            }
            None => self.heads[slot_id] = Some(new_id),
        }

        new_id
    }

    pub fn move_to_another_slot(&mut self, uid : usize, new_slot_id : usize) {
        let (prev, next, slot_id) = {
            let cell = self.get_cell(uid).unwrap();
            (cell.prev, cell.next, cell.slot_id)
        };

        match prev {
            Some(prev) => {
                let prev_cell = self.get_cell_mut(prev).unwrap();
                prev_cell.next = next;
            },
            None => self.heads[slot_id] = next,
        }

        if let Some(next) = next {
            self.get_cell_mut(next).unwrap().prev = prev;
        }

        if let Some(head) = self.heads[new_slot_id] {
            self.get_cell_mut(head).unwrap().prev = Some(uid);
        }

        self.heads[new_slot_id] = Some(uid);
    }

    #[inline]
    fn get_cell<'a>(&'a self, uid : usize) -> Option<&'a Cell<T>> {
        self.cells.get(uid).and_then(|x| x.as_ref())
    }

    #[inline]
    pub fn get<'a>(&'a self, uid : usize) -> Option<&'a T> {
        self.get_cell(uid).map(|x| &x.val)
    }

    #[inline]
    fn get_cell_mut<'a>(&'a mut self, uid : usize) -> Option<&'a mut Cell<T>> {
        self.cells.get_mut(uid).and_then(|x| x.as_mut())
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, uid : usize) -> Option<&'a mut T> {
        self.get_cell_mut(uid).map(|x| &mut x.val)
    }

    pub fn remove(&mut self, uid : usize) -> T {
        let cell = self.cells.get_mut(uid).unwrap().take().unwrap();
        
        match cell.prev {
            // An ordinary cell
            Some(prev) => self.get_cell_mut(prev).unwrap().next = cell.next,
            // Head (because it has no previous element)
            None => self.heads[cell.slot_id] = cell.next,
        }

        if let Some(next) = cell.next { self.get_cell_mut(next).unwrap().prev = cell.prev }

        self.vacant.push(uid);

        cell.val
    }

    #[inline]
    pub fn retain<F : Fn(&T) -> bool>(&mut self, f : F) {
        let len = self.cells.len();
        for i in 0..len {
            let stay = 
                match self.get(i) {
                    Some(val) => f(&val),
                    None => continue,
                }
            ;

            if !stay { self.remove(i); }
        }
    }
  
/*
    #[inline]
    pub fn iter(&self) -> Iter<T> { Iter { current : self.head, container : self } }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> { IterMut { current : self.head, container : self } }
*/

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cells.capacity()
    }
}

/*
pub struct Iter<'a, T : 'a> {
    current : Option<usize>,
    container : &'a ConsistentLinearChunk<T>,
}

impl<'a, T : 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        let current = match self.current { Some(x) => x, None => return None };

        let cell = self.container.get_cell(current).unwrap();

        self.current = cell.next;
        Some(&cell.val)
    }
}

pub struct IterMut<'a, T : 'a> {
    current : Option<usize>,
    container : &'a mut ConsistentLinearChunk<T>,
}

impl<'a, T : 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        use std::mem;

        let current = match self.current { Some(x) => x, None => return None };

        let cell = self.container.get_cell_mut(current).unwrap();
        
        self.current = cell.next;
        unsafe { Some(mem::transmute::<_, &'a mut T>(&mut cell.val)) }
    }
}
*/
