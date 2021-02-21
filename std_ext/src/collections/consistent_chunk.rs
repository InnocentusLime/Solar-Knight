/// A linked list which stores its memory continiously
/// As a bonus, it's elements have stable IDs.
/// This structure is used to store arrays of constantly
/// appearing and disappearing entities. E.g. bullets
pub struct ConsistentChunk<T> {
    vals : Vec<Option<T>>,
    vacant : Vec<usize>,
}

impl<T> ConsistentChunk<T> {
    pub fn new() -> Self {
        ConsistentChunk {
            vals : Vec::new(),
            vacant : Vec::new(),
        }
    }

    pub fn with_capacity(cap : usize) -> Self {
        ConsistentChunk {
            vals : Vec::with_capacity(cap),
            vacant : Vec::with_capacity(cap),
        }
    }

    pub fn push(&mut self, x : T) -> usize {
        let new_id = 
            match self.vacant.pop() {
                Some(vacant) => {
                    self.vals[vacant] = Some(x);
                    vacant
                },
                None => {
                    self.vals.push(Some(x));
                    self.vals.len() - 1
                },
            }
        ;
        
        new_id
    }

    #[inline]
    pub fn get<'a>(&'a self, uid : usize) -> Option<&'a T> {
        self.vals.get(uid).and_then(|x| x.as_ref())
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, uid : usize) -> Option<&'a mut T> {
        self.vals.get_mut(uid).and_then(|x| x.as_mut())
    }

    pub fn remove(&mut self, uid : usize) -> T {
        let val = self.vals[uid].take();

        self.vacant.push(uid);

        val.unwrap()
    }

    #[inline]
    pub fn retain<F : Fn(&T) -> bool>(&mut self, f : F) {
        let len = self.vals.len();
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
   
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> { self.vals.iter().filter_map(|x| x.as_ref()) }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> { self.vals.iter_mut().filter_map(|x| x.as_mut()) }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.vals.capacity()
    }
}
